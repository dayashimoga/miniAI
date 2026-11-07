use std::sync::atomic::AtomicBool;

use miniai_video::adapter::VideoAdapter;
use miniai_video::buffer_pool::SharedFrameBufferPool;
use miniai_video::types::{
    BufferPoolConfig, CameraMotion, NodeExecutionStatus, VideoSynthesisConfig, WorkflowData,
    WorkflowNode, WorkflowNodeType,
};
use miniai_video::workflow::{MediaWorkflowGraph, WorkflowExecutor};
use miniai_video::VideoError;

#[test]
fn test_linear_multimodal_workflow() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let executor = WorkflowExecutor::new(pool);

    let mut graph = MediaWorkflowGraph::new("linear_media_pipeline");

    // 1. Scriptwriter Node (LLM)
    graph.add_node(WorkflowNode::new(
        "scriptwriter",
        "LLM Storyboard Scriptwriter",
        WorkflowNodeType::LlmPrompt {
            prompt: "Sunset over ocean waves".to_string(),
            max_tokens: 256,
        },
    ));

    // 2. Concept Artist Node (ImageGen)
    graph.add_node(WorkflowNode::new(
        "concept_artist",
        "Concept Image Generator",
        WorkflowNodeType::ImageGen {
            prompt: "Golden hour ocean waves".to_string(),
            width: 64,
            height: 64,
        },
    ));

    // 3. Video Animator Node (VideoSynthesis)
    graph.add_node(WorkflowNode::new(
        "animator",
        "Camera Motion Video Animator",
        WorkflowNodeType::VideoSynthesis {
            motion: CameraMotion::PanRight,
            duration_secs: 0.5,
        },
    ));

    // 4. Video Inspector Node (VideoAnalysis)
    graph.add_node(WorkflowNode::new(
        "inspector",
        "Temporal Scene Inspector",
        WorkflowNodeType::VideoAnalysis,
    ));

    // Edges
    graph.add_edge("scriptwriter", "concept_artist");
    graph.add_edge("concept_artist", "animator");
    graph.add_edge("animator", "inspector");

    let result = executor
        .execute(&graph, None)
        .expect("Execute linear workflow");

    assert_eq!(result.graph_id, "linear_media_pipeline");
    assert_eq!(result.outputs.len(), 4);
    assert_eq!(result.telemetry.len(), 4);

    for telem in &result.telemetry {
        assert_eq!(telem.status, NodeExecutionStatus::Completed);
        assert!(telem.error_message.is_none());
    }

    // Verify final node output is JSON analysis
    let final_output = result.outputs.get("inspector").expect("inspector output");
    assert!(matches!(final_output, WorkflowData::Json(_)));
}

#[test]
fn test_branching_multimodal_dag_pipeline() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let executor = WorkflowExecutor::new(pool);

    let mut graph = MediaWorkflowGraph::new("branching_multimodal_dag");

    // Node 1: LLM Prompt
    graph.add_node(WorkflowNode::new(
        "llm_script",
        "LLM Director",
        WorkflowNodeType::LlmPrompt {
            prompt: "Hero enters ancient mystical forest".to_string(),
            max_tokens: 128,
        },
    ));

    // Node 2: Image Keyframe
    graph.add_node(WorkflowNode::new(
        "keyframe_gen",
        "Keyframe Generator",
        WorkflowNodeType::ImageGen {
            prompt: "Ancient glowing forest".to_string(),
            width: 64,
            height: 64,
        },
    ));

    // Node 3: Voiceover (TTS)
    graph.add_node(WorkflowNode::new(
        "voiceover",
        "Voiceover Actor",
        WorkflowNodeType::SpeechTts {
            voice: "Onyx".to_string(),
            speed: 1.0,
        },
    ));

    // Node 4: Soundtrack (Music)
    graph.add_node(WorkflowNode::new(
        "soundtrack",
        "Background Music Composer",
        WorkflowNodeType::MusicGen {
            mood: "Epic orchestral mystical".to_string(),
            duration_secs: 1.0,
        },
    ));

    // Node 5: Video Animation
    graph.add_node(WorkflowNode::new(
        "video_anim",
        "Temporal Scene Animator",
        WorkflowNodeType::VideoSynthesis {
            motion: CameraMotion::ZoomIn,
            duration_secs: 0.5,
        },
    ));

    // Node 6: Multiplexer
    graph.add_node(WorkflowNode::new(
        "muxer",
        "Audio Video Multiplexer",
        WorkflowNodeType::AudioVideoMux,
    ));

    // Edges
    // LLM branches into Image, Voiceover, and Music
    graph.add_edge("llm_script", "keyframe_gen");
    graph.add_edge("llm_script", "voiceover");
    graph.add_edge("llm_script", "soundtrack");

    // Image feeds Video
    graph.add_edge("keyframe_gen", "video_anim");

    // Video, Voiceover, Music converge into Muxer
    graph.add_edge("video_anim", "muxer");
    graph.add_edge("voiceover", "muxer");
    graph.add_edge("soundtrack", "muxer");

    let result = executor
        .execute(&graph, None)
        .expect("Execute branching DAG");

    assert_eq!(result.outputs.len(), 6);
    let mux_out = result.outputs.get("muxer").expect("muxer output");
    if let WorkflowData::Video {
        metadata,
        video_bytes,
    } = mux_out
    {
        assert!(metadata.total_frames >= 2);
        assert!(!video_bytes.is_empty());
    } else {
        panic!("Muxer output must be Video");
    }
}

#[test]
fn test_dag_cycle_detection() {
    let mut graph = MediaWorkflowGraph::new("cycle_graph");

    graph.add_node(WorkflowNode::new(
        "node_a",
        "Node A",
        WorkflowNodeType::LlmPrompt {
            prompt: "A".to_string(),
            max_tokens: 32,
        },
    ));
    graph.add_node(WorkflowNode::new(
        "node_b",
        "Node B",
        WorkflowNodeType::SpeechTts {
            voice: "Alloy".to_string(),
            speed: 1.0,
        },
    ));
    graph.add_node(WorkflowNode::new(
        "node_c",
        "Node C",
        WorkflowNodeType::SpeechAsr,
    ));

    // Create cycle: A -> B -> C -> A
    graph.add_edge("node_a", "node_b");
    graph.add_edge("node_b", "node_c");
    graph.add_edge("node_c", "node_a");

    let err = graph.validate_and_sort().expect_err("Should detect cycle");
    assert!(matches!(err, VideoError::WorkflowCycleDetected(_)));
}

#[test]
fn test_dag_missing_node_rejection() {
    let mut graph = MediaWorkflowGraph::new("missing_node_graph");
    graph.add_node(WorkflowNode::new(
        "existing_node",
        "Existing",
        WorkflowNodeType::SpeechAsr,
    ));
    graph.add_edge("existing_node", "non_existent_node");

    let err = graph
        .validate_and_sort()
        .expect_err("Should fail for missing node");
    assert!(matches!(err, VideoError::WorkflowNodeNotFound { .. }));
}

#[test]
fn test_workflow_cooperative_cancellation() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let executor = WorkflowExecutor::new(pool);

    let mut graph = MediaWorkflowGraph::new("cancel_graph");
    graph.add_node(WorkflowNode::new(
        "node_1",
        "Step 1",
        WorkflowNodeType::LlmPrompt {
            prompt: "P".to_string(),
            max_tokens: 32,
        },
    ));

    let token = AtomicBool::new(true); // Pre-cancelled
    let err = executor
        .execute(&graph, Some(&token))
        .expect_err("Should cancel");
    assert!(matches!(err, VideoError::Cancelled));
}

#[test]
fn test_vision_and_asr_and_custom_nodes() {
    let pool = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let executor = WorkflowExecutor::new(pool);

    let mut graph = MediaWorkflowGraph::new("multimodal_mix");

    graph.add_node(WorkflowNode::new(
        "img_src",
        "Image Source",
        WorkflowNodeType::ImageGen {
            prompt: "Meadow".to_string(),
            width: 32,
            height: 32,
        },
    ));

    graph.add_node(WorkflowNode::new(
        "vision_node",
        "Vision Inspector",
        WorkflowNodeType::VisionAnalyze,
    ));

    graph.add_node(WorkflowNode::new(
        "voice_node",
        "Voice Source",
        WorkflowNodeType::SpeechTts {
            voice: "Nova".to_string(),
            speed: 1.0,
        },
    ));

    graph.add_node(WorkflowNode::new(
        "asr_node",
        "ASR Transcriber",
        WorkflowNodeType::SpeechAsr,
    ));

    graph.add_node(WorkflowNode::new(
        "custom_node",
        "Custom Filter",
        WorkflowNodeType::Custom {
            kind: "metadata_tagger".to_string(),
            config: serde_json::json!({ "tag": "prod_v1" }),
        },
    ));

    graph.add_edge("img_src", "vision_node");
    graph.add_edge("voice_node", "asr_node");
    graph.add_edge("vision_node", "custom_node");

    let result = executor.execute(&graph, None).expect("Run mix");
    assert_eq!(result.outputs.len(), 5);

    let vision_out = result.outputs.get("vision_node").expect("vision");
    assert!(matches!(vision_out, WorkflowData::Text(_)));

    let asr_out = result.outputs.get("asr_node").expect("asr");
    assert!(matches!(asr_out, WorkflowData::Text(_)));

    let custom_out = result.outputs.get("custom_node").expect("custom");
    assert!(matches!(custom_out, WorkflowData::Json(_)));
}

#[test]
fn test_video_adapter_synthesis_and_workflow_integration() {
    let adapter = VideoAdapter::new("video-cpu-adapter");

    // Test synthesis API on adapter
    let config = VideoSynthesisConfig {
        prompt: "Adapter synthesis test".to_string(),
        motion: CameraMotion::PanLeft,
        width: 32,
        height: 32,
        duration_secs: 0.5,
        fps: 6.0,
        seed: Some(123),
        start_frame: None,
        end_frame: None,
        generate_audio: true,
    };
    let synth_res = adapter.synthesize_video(&config).expect("adapter synth");
    assert_eq!(synth_res.metadata.width, 32);

    // Test workflow execution on adapter
    let mut graph = MediaWorkflowGraph::new("adapter_workflow");
    graph.add_node(WorkflowNode::new(
        "step_1",
        "Script",
        WorkflowNodeType::LlmPrompt {
            prompt: "Test".to_string(),
            max_tokens: 16,
        },
    ));
    let wf_res = adapter
        .execute_workflow(&graph, None)
        .expect("adapter workflow");
    assert_eq!(wf_res.outputs.len(), 1);
}

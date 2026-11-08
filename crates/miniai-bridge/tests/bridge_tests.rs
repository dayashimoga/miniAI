use std::ffi::{CStr, CString};
use std::ptr;
use tempfile::tempdir;

use miniai_bridge::contracts::{BridgeRequest, BridgeResponse};
use miniai_bridge::engine::BridgeEngine;
use miniai_bridge::ffi::{
    miniai_bridge_dispatch, miniai_bridge_free, miniai_bridge_init, miniai_bridge_string_free,
};

#[test]
fn test_bridge_engine_in_memory_lifecycle() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine should initialize");

    // 1. Hardware Probe
    let res = engine.dispatch(BridgeRequest::ProbeHardware);
    assert!(res.success);
    assert!(res.data.is_some());
    let hw_data = res.data.unwrap();
    assert!(hw_data.get("logical_cpu_cores").is_some());

    // 2. Privacy Mode Read and Toggle
    let p_res = engine.dispatch(BridgeRequest::GetPrivacyStatus);
    assert!(p_res.success);
    let p_state = p_res.data.unwrap();
    assert_eq!(p_state.get("local_only_mode").unwrap(), true);

    let set_res = engine.dispatch(BridgeRequest::SetPrivacyMode { local_only: false });
    assert!(set_res.success);
    let updated_state = set_res.data.unwrap();
    assert_eq!(updated_state.get("local_only_mode").unwrap(), false);

    // 3. Chat Execution
    let chat_res = engine.dispatch(BridgeRequest::ExecuteChat {
        session_id: "test-session-01".to_string(),
        prompt: "what are your capabilities".to_string(),
        model_id: Some("llama-3.2-1b-instruct-q4".to_string()),
        messages: None,
        temperature: None,
        max_tokens: None,
        system_prompt: None,
    });
    assert!(chat_res.success);
    let chat_data = chat_res.data.unwrap();
    assert_eq!(chat_data.get("session_id").unwrap(), "test-session-01");
    assert_eq!(chat_data.get("status").unwrap(), "completed");

    // 4. Knowledge Search
    let rag_res = engine.dispatch(BridgeRequest::SearchKnowledge {
        query: "admission guard CPU fallback".to_string(),
        limit: 5,
    });
    assert!(rag_res.success);
    let rag_data = rag_res.data.unwrap();
    assert_eq!(
        rag_data.get("query").unwrap(),
        "admission guard CPU fallback"
    );
    let results = rag_data.get("results").unwrap().as_array().unwrap();
    assert!(
        !results.is_empty(),
        "knowledge search should return grounded hits"
    );
    assert!(results[0].get("citation_tag").is_some());

    // 5. Audit & Diagnostic Events
    let events_res = engine.dispatch(BridgeRequest::PollEvents);
    assert!(events_res.success);
    let audit_res = engine.dispatch(BridgeRequest::GetAuditLedger { limit: 10 });
    assert!(audit_res.success);
}

#[test]
fn test_bridge_engine_on_disk_persistence() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("bridge_storage.db");

    {
        let engine = BridgeEngine::open(&db_path).expect("open on-disk db");
        let res = engine.dispatch(BridgeRequest::SetPrivacyMode { local_only: false });
        assert!(res.success);
    }

    // Reopen and verify persistence
    {
        let engine = BridgeEngine::open(&db_path).expect("reopen on-disk db");
        let events_res = engine.dispatch(BridgeRequest::PollEvents);
        assert!(events_res.success);
        let events = events_res.data.unwrap();
        assert!(!events.as_array().unwrap().is_empty());
    }
}

#[test]
fn test_bridge_json_dispatch_protocol() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");

    let req_json = r#"{"action":"probe_hardware"}"#;
    let resp_str = engine.dispatch_json(req_json);
    let parsed: BridgeResponse = serde_json::from_str(&resp_str).expect("valid response JSON");
    assert!(parsed.success);
    assert!(parsed.data.is_some());

    // Invalid JSON request
    let bad_json = r#"{"action":"unknown_action_xyz"}"#;
    let bad_resp_str = engine.dispatch_json(bad_json);
    let bad_parsed: BridgeResponse = serde_json::from_str(&bad_resp_str).expect("valid error JSON");
    assert!(!bad_parsed.success);
    assert_eq!(bad_parsed.error.unwrap().code, "INVALID_JSON_REQUEST");
}

#[test]
fn test_bridge_run_doctor() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");
    let resp = engine.dispatch(BridgeRequest::RunDoctor);
    assert!(resp.success);
    let data = resp.data.expect("doctor data");
    assert_eq!(data["overall_status"], "healthy");
    assert_eq!(data["privacy"]["local_only_mode"], true);
    assert_eq!(data["storage"]["is_healthy"], true);
    assert!(data["hardware"]["logical_cpu_cores"].as_u64().unwrap() > 0);
    assert_eq!(data["daemon"]["loopback_enforced"], true);
}

#[test]
fn test_bridge_telemetry_and_model_unloading() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");

    // 1. Initial telemetry before any chat
    let res = engine.dispatch(BridgeRequest::GetTelemetry);
    assert!(res.success);
    let telem = res.data.unwrap();
    assert_eq!(telem["active_model"], serde_json::Value::Null);
    assert_eq!(telem["total_chat_requests"], 0);

    // 2. Execute chat -> sets active model and increments telemetry
    let chat_res = engine.dispatch(BridgeRequest::ExecuteChat {
        session_id: "obs-session-01".to_string(),
        prompt: "measure throughput and latency".to_string(),
        model_id: Some("llama-3.2-1b-instruct-q4".to_string()),
        messages: None,
        temperature: None,
        max_tokens: None,
        system_prompt: None,
    });
    assert!(chat_res.success);
    let chat_data = chat_res.data.unwrap();
    assert!(chat_data["telemetry"]["ttft_ms"].as_f64().unwrap() > 0.0);
    assert!(
        chat_data["telemetry"]["tokens_per_second"]
            .as_f64()
            .unwrap()
            > 0.0
    );

    // 3. Verify telemetry updated
    let telem2 = engine.dispatch(BridgeRequest::GetTelemetry).data.unwrap();
    assert_eq!(telem2["active_model"], "llama-3.2-1b-instruct-q4");
    assert_eq!(telem2["total_chat_requests"], 1);
    assert_eq!(
        telem2["total_tokens_generated"],
        chat_data["telemetry"]["tokens_generated"]
    );

    // 4. Unload model
    let unload_res = engine.dispatch(BridgeRequest::UnloadModel { model_id: None });
    assert!(unload_res.success);
    let unload_data = unload_res.data.unwrap();
    assert_eq!(unload_data["status"], "unloaded");
    assert!(unload_data["freed_memory_mb"].as_f64().unwrap() > 0.0);

    // 5. Verify telemetry shows unloaded
    let telem3 = engine.dispatch(BridgeRequest::GetTelemetry).data.unwrap();
    assert_eq!(telem3["active_model"], serde_json::Value::Null);
}

#[test]
fn test_bridge_export_redacted_diagnostics() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");
    let resp = engine.dispatch(BridgeRequest::ExportRedactedDiagnostics);
    assert!(resp.success);
    let data = resp.data.expect("diagnostic bundle");
    assert!(data.get("timestamp").is_some());
    assert!(data.get("platform").is_some());
    assert_eq!(data["privacy_mode"], "Local-Only Enforced");
}

#[test]
fn test_c_ffi_lifecycle_and_dispatch() {
    unsafe {
        // 1. Initialize with in-memory storage (NULL path)
        let engine_ptr = miniai_bridge_init(ptr::null());
        assert!(!engine_ptr.is_null());

        // 2. Dispatch valid request
        let req_cstr = CString::new(r#"{"action":"probe_hardware"}"#).unwrap();
        let resp_ptr = miniai_bridge_dispatch(engine_ptr, req_cstr.as_ptr());
        assert!(!resp_ptr.is_null());

        let resp_cstr = CStr::from_ptr(resp_ptr);
        let resp_str = resp_cstr.to_str().expect("valid UTF-8 response");
        let parsed: BridgeResponse = serde_json::from_str(resp_str).expect("valid JSON response");
        assert!(parsed.success);
        assert!(parsed.data.is_some());

        // Free response string
        miniai_bridge_string_free(resp_ptr);

        // 3. Test null-pointer safety
        let null_resp_ptr = miniai_bridge_dispatch(ptr::null_mut(), req_cstr.as_ptr());
        assert!(!null_resp_ptr.is_null());
        let null_resp_str = CStr::from_ptr(null_resp_ptr).to_str().unwrap();
        assert!(null_resp_str.contains("NULL_POINTER"));
        miniai_bridge_string_free(null_resp_ptr);

        // 4. Free engine
        miniai_bridge_free(engine_ptr);
    }
}

#[test]
fn test_bridge_memory_tiers_crud() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");

    // 1. Initial memories should be empty
    let list_res = engine.dispatch(BridgeRequest::ListMemories);
    assert!(list_res.success);
    let data = list_res.data.unwrap();
    assert_eq!(data["semantic_facts"].as_array().unwrap().len(), 0);

    // 2. Store user preference
    let store_res = engine.dispatch(BridgeRequest::StoreMemory {
        memory_type: "semantic".to_string(),
        content: "User prefers local-first offline execution".to_string(),
        metadata: None,
    });
    assert!(store_res.success);

    // 3. Verify memory appears in ListMemories
    let list_res2 = engine.dispatch(BridgeRequest::ListMemories);
    assert!(list_res2.success);
    let data2 = list_res2.data.unwrap();
    assert_eq!(data2["semantic_facts"].as_array().unwrap().len(), 1);

    // 4. Clear memories
    let clear_res = engine.dispatch(BridgeRequest::ClearMemories);
    assert!(clear_res.success);
    let clear_data = clear_res.data.unwrap();
    assert_eq!(clear_data["status"], "cleared");

    // 5. Verify empty again
    let list_res3 = engine.dispatch(BridgeRequest::ListMemories);
    let data3 = list_res3.data.unwrap();
    assert_eq!(data3["semantic_facts"].as_array().unwrap().len(), 0);
}

#[test]
fn test_bridge_stage15_mcp_swarm_and_npu() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");

    // 1. List MCP Tools
    let mcp_res = engine.dispatch(BridgeRequest::ListMcpTools);
    assert!(mcp_res.success);
    let mcp_data = mcp_res.data.unwrap();
    assert_eq!(mcp_data["mcp_enabled"], true);
    let tools = mcp_data["tools"].as_array().unwrap();
    assert!(tools.len() >= 3);

    // 2. Swarm Status
    let swarm_res = engine.dispatch(BridgeRequest::GetSwarmStatus);
    assert!(swarm_res.success);
    let swarm_data = swarm_res.data.unwrap();
    assert_eq!(swarm_data["p2p_transport"], "mTLS");

    // 3. NPU Backends
    let npu_res = engine.dispatch(BridgeRequest::GetNpuBackends);
    assert!(npu_res.success);
    let npu_data = npu_res.data.unwrap();
    assert_eq!(npu_data["backends"][0]["name"], "CPU Baseline");
}

#[test]
fn test_bridge_chat_no_canned_boilerplate_and_neural_execution() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");
    let res = engine.dispatch(BridgeRequest::ExecuteChat {
        session_id: "test-real-chat".to_string(),
        prompt: "how to build a picnic tent".to_string(),
        model_id: Some("smollm2-135m-instruct-q4".to_string()),
        messages: None,
        temperature: Some(0.0),
        max_tokens: Some(32),
        system_prompt: None,
    });
    assert!(res.success);
    let data = res.data.unwrap();
    let text = data["text"].as_str().unwrap();
    assert!(
        !text.contains("MiniAI analyzed query"),
        "Must never return canned boilerplate template, got: {}",
        text
    );
    assert!(!text.is_empty());
}

#[test]
fn test_bridge_stage17_streaming_model_downloader_and_local_import() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");

    // 1. In Local-Only Privacy Mode, remote download must be blocked
    let block_res = engine.dispatch(BridgeRequest::StartModelDownload {
        model_id: "llama-3.2-1b-instruct-q4".to_string(),
        source_url: "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
        expected_sha256: Some("6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83".to_string()),
        expected_size_bytes: Some(807694464),
    });
    assert!(!block_res.success);
    assert_eq!(block_res.error.unwrap().code, "PRIVACY_BLOCKED");

    // 2. Disable Local-Only Privacy Mode to permit download
    let p_res = engine.dispatch(BridgeRequest::SetPrivacyMode { local_only: false });
    assert!(p_res.success);

    // 3. Start download
    let start_res = engine.dispatch(BridgeRequest::StartModelDownload {
        model_id: "llama-3.2-1b-instruct-q4".to_string(),
        source_url: "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
        expected_sha256: Some("6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83".to_string()),
        expected_size_bytes: Some(807694464),
    });
    assert!(start_res.success);
    let start_data = start_res.data.unwrap();
    assert_eq!(start_data["model_id"], "llama-3.2-1b-instruct-q4");

    // 4. Query progress
    let prog_res = engine.dispatch(BridgeRequest::GetModelDownloadProgress {
        model_id: "llama-3.2-1b-instruct-q4".to_string(),
    });
    assert!(prog_res.success);

    // 5. Cancel download
    let cancel_res = engine.dispatch(BridgeRequest::CancelModelDownload {
        model_id: "llama-3.2-1b-instruct-q4".to_string(),
    });
    assert!(cancel_res.success);
    assert_eq!(cancel_res.data.unwrap()["status"], "cancelled");

    // 6. Test ImportLocalModel with existing smollm2 model
    let smol_path = "models/smollm2-135m-instruct-q4.gguf";
    if std::path::Path::new(smol_path).is_file() {
        let import_res = engine.dispatch(BridgeRequest::ImportLocalModel {
            source_path: smol_path.to_string(),
            model_id: "smollm2-imported-test".to_string(),
            name: Some("SmolLM2 Imported Test".to_string()),
            format: Some("gguf".to_string()),
            expected_sha256: Some(
                "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d".to_string(),
            ),
            expected_size_bytes: Some(105454432),
        });
        assert!(import_res.success);
        let import_data = import_res.data.unwrap();
        assert_eq!(import_data["status"], "verified");
        assert_eq!(
            import_data["sha256"],
            "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d"
        );
    }
}

#[test]
fn test_bridge_stage19_multimodal_studio_dispatch() {
    let engine = BridgeEngine::new_in_memory().expect("in-memory engine");

    // 1. GenerateImage
    let img_res = engine.dispatch(BridgeRequest::GenerateImage {
        prompt: "A neon cyberpunk cityscape with flying cars".to_string(),
        negative_prompt: Some("blurry, low quality".to_string()),
        mode: Some("text_to_image".to_string()),
        width: Some(128),
        height: Some(128),
        steps: Some(5),
        guidance_scale: Some(7.5),
        seed: Some(101),
    });
    assert!(img_res.success, "GenerateImage failed: {:?}", img_res.error);
    let img_data = img_res.data.unwrap();
    assert!(img_data.get("base64_png").is_some());
    assert!(img_data.get("sha256").is_some());
    assert_eq!(img_data.get("width").unwrap(), 128);
    assert_eq!(img_data.get("height").unwrap(), 128);

    // 2. AnalyzeVision
    let vis_res = engine.dispatch(BridgeRequest::AnalyzeVision {
        image_base64: None,
        prompt: Some("What text appears in this frame?".to_string()),
        mode: Some("ocr_and_vqa".to_string()),
        width: Some(256),
        height: Some(256),
    });
    assert!(vis_res.success, "AnalyzeVision failed: {:?}", vis_res.error);
    let vis_data = vis_res.data.unwrap();
    assert!(vis_data.get("image_sha256").is_some());
    assert!(vis_data.get("ocr_boxes").is_some());
    assert!(vis_data.get("vqa_answer").is_some());

    // 3. TranscribeSpeech
    let asr_res = engine.dispatch(BridgeRequest::TranscribeSpeech {
        audio_base64: None,
        sample_rate: Some(16000),
        language: Some("en".to_string()),
    });
    assert!(
        asr_res.success,
        "TranscribeSpeech failed: {:?}",
        asr_res.error
    );
    let asr_data = asr_res.data.unwrap();
    assert!(asr_data.get("text").is_some());
    assert!(asr_data.get("confidence").is_some());

    // 4. SynthesizeSpeech
    let tts_res = engine.dispatch(BridgeRequest::SynthesizeSpeech {
        text: "MiniAI is running fully offline with zero telemetry leakage.".to_string(),
        voice_id: Some("alloy".to_string()),
        pitch: Some(0.0),
        speed: Some(1.0),
        sample_rate: Some(22050),
    });
    assert!(
        tts_res.success,
        "SynthesizeSpeech failed: {:?}",
        tts_res.error
    );
    let tts_data = tts_res.data.unwrap();
    assert!(tts_data.get("audio_base64").is_some());
    assert!(tts_data.get("duration_secs").is_some());
    assert!(tts_data.get("sha256").is_some());

    // 5. SynthesizeMusic
    let music_res = engine.dispatch(BridgeRequest::SynthesizeMusic {
        genre: Some("Lo-fi chill beats".to_string()),
        tempo_bpm: Some(100),
        duration_secs: Some(2.0),
        key_signature: Some("C Major".to_string()),
        seed: Some(42),
    });
    assert!(
        music_res.success,
        "SynthesizeMusic failed: {:?}",
        music_res.error
    );
    let music_data = music_res.data.unwrap();
    assert!(music_data.get("audio_base64").is_some());
    assert!(music_data.get("sha256").is_some());
    assert_eq!(music_data.get("tempo_bpm").unwrap(), 100);

    // 6. SynthesizeVideo
    let video_res = engine.dispatch(BridgeRequest::SynthesizeVideo {
        prompt: "A drone flying over snow-covered mountains".to_string(),
        width: Some(160),
        height: Some(120),
        fps: Some(10.0),
        duration_secs: Some(1.0),
        seed: Some(99),
    });
    assert!(
        video_res.success,
        "SynthesizeVideo failed: {:?}",
        video_res.error
    );
    let video_data = video_res.data.unwrap();
    assert!(video_data.get("video_id").is_some());
    assert!(video_data.get("sha256").is_some());
    assert!(video_data.get("total_frames").is_some());

    // 7. ExecuteTrainingJob
    let train_res = engine.dispatch(BridgeRequest::ExecuteTrainingJob {
        task_type: "lora_instruct".to_string(),
        base_model: "llama-3.2-1b".to_string(),
        dataset_name: "alpaca_clean_sample".to_string(),
        epochs: Some(2),
        learning_rate: Some(0.0002),
        batch_size: Some(4),
        lora_rank: Some(16),
    });
    assert!(
        train_res.success,
        "ExecuteTrainingJob failed: {:?}",
        train_res.error
    );
    let train_data = train_res.data.unwrap();
    assert_eq!(train_data.get("status").unwrap(), "completed");
    assert_eq!(train_data.get("lora_rank").unwrap(), 16);
    assert!(train_data.get("history").is_some());
}

#[test]
fn test_bridge_stage20_power_governor_and_kv_cache() {
    let engine = BridgeEngine::new_in_memory().expect("failed to create engine");

    // 1. Get initial PowerProfile
    let p_res = engine.dispatch(BridgeRequest::GetPowerProfile);
    assert!(p_res.success, "{:?}", p_res.error);
    let p_data = p_res.data.unwrap();
    assert_eq!(p_data.get("is_throttled").unwrap(), false);
    assert_eq!(p_data.get("thermal_state").unwrap(), "nominal");
    assert_eq!(p_data.get("max_context_tokens").unwrap(), 4096);

    // 2. Enable battery saving mode
    let set_res = engine.dispatch(BridgeRequest::SetPowerProfile {
        battery_saving: Some(true),
        simulated_thermal: None,
        simulated_power: None,
    });
    assert!(set_res.success, "{:?}", set_res.error);
    let set_data = set_res.data.unwrap();
    assert_eq!(set_data.get("is_throttled").unwrap(), true);
    assert_eq!(set_data.get("battery_saving_active").unwrap(), true);
    assert_eq!(set_data.get("max_context_tokens").unwrap(), 1024);

    // 3. Set critical thermal state
    let th_res = engine.dispatch(BridgeRequest::SetPowerProfile {
        battery_saving: Some(false),
        simulated_thermal: Some(miniai_core::ThermalState::Critical),
        simulated_power: None,
    });
    assert!(th_res.success, "{:?}", th_res.error);
    let th_data = th_res.data.unwrap();
    assert_eq!(th_data.get("is_throttled").unwrap(), true);
    assert_eq!(th_data.get("thermal_state").unwrap(), "critical");

    // 4. Low battery level (15%) discharging forces CPU recommendation
    let bat_res = engine.dispatch(BridgeRequest::SetPowerProfile {
        battery_saving: Some(false),
        simulated_thermal: Some(miniai_core::ThermalState::Nominal),
        simulated_power: Some(miniai_core::PowerState::Battery {
            level_percent: 15,
            is_charging: false,
        }),
    });
    assert!(bat_res.success, "{:?}", bat_res.error);
    let bat_data = bat_res.data.unwrap();
    assert_eq!(bat_data.get("recommended_backend").unwrap(), "cpu");

    // 5. Select Accelerator Backend (Cpu and Cuda)
    let sel_cpu = engine.dispatch(BridgeRequest::SelectAcceleratorBackend {
        backend: miniai_core::AcceleratorBackend::Cpu,
    });
    assert!(sel_cpu.success);
    assert_eq!(sel_cpu.data.unwrap().get("is_cpu").unwrap(), true);

    let sel_cuda = engine.dispatch(BridgeRequest::SelectAcceleratorBackend {
        backend: miniai_core::AcceleratorBackend::Cuda,
    });
    assert!(sel_cuda.success);

    // 6. Reset KV cache
    let rst_res = engine.dispatch(BridgeRequest::ResetKvCache);
    assert!(rst_res.success);
    assert_eq!(
        rst_res.data.unwrap().get("status").unwrap(),
        "kv_cache_reset"
    );
}

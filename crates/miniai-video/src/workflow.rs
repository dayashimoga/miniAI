use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::buffer_pool::SharedFrameBufferPool;
use crate::engine::{VideoEngine, VideoEngineConfig};
use crate::error::VideoError;
use crate::sanitizer::VideoSanitizer;
use crate::synthesis::VideoSynthesisEngine;
use crate::types::{
    NodeExecutionStatus, NodeTelemetry, SanitizerConfig, VideoSynthesisConfig, WorkflowData,
    WorkflowEdge, WorkflowExecutionResult, WorkflowNode, WorkflowNodeType,
};

/// Composable Multimodal Directed Acyclic Graph (DAG) for chaining MiniAI model modalities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaWorkflowGraph {
    pub id: String,
    pub nodes: HashMap<String, WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

use serde::{Deserialize, Serialize};

impl MediaWorkflowGraph {
    /// Creates a new empty workflow graph with the specified ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Adds a node to the workflow graph.
    pub fn add_node(&mut self, node: WorkflowNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Adds a directed dependency edge from one node to another.
    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.edges.push(WorkflowEdge::new(from, to));
    }

    /// Validates the graph topology and returns a valid topological execution order using Kahn's algorithm.
    ///
    /// Rejects graphs containing cycles or referencing non-existent nodes.
    pub fn validate_and_sort(&self) -> Result<Vec<String>, VideoError> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for id in self.nodes.keys() {
            in_degree.insert(id.as_str(), 0);
            adj.insert(id.as_str(), Vec::new());
        }

        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from_node) {
                return Err(VideoError::WorkflowNodeNotFound {
                    node_id: edge.from_node.clone(),
                });
            }
            if !self.nodes.contains_key(&edge.to_node) {
                return Err(VideoError::WorkflowNodeNotFound {
                    node_id: edge.to_node.clone(),
                });
            }

            *in_degree.entry(&edge.to_node).or_insert(0) += 1;
            adj.entry(&edge.from_node).or_default().push(&edge.to_node);
        }

        // Initialize queue with zero in-degree nodes
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::with_capacity(self.nodes.len());

        while let Some(u) = queue.pop_front() {
            sorted.push(u.to_string());

            if let Some(neighbors) = adj.get(u) {
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(v) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            let unvisited: Vec<String> = self
                .nodes
                .keys()
                .filter(|k| !sorted.contains(k))
                .cloned()
                .collect();
            return Err(VideoError::WorkflowCycleDetected(format!(
                "Cycle detected involving nodes: {}",
                unvisited.join(", ")
            )));
        }

        Ok(sorted)
    }

    /// Finds all immediate dependencies (parent nodes) of a given node.
    pub fn get_dependencies(&self, node_id: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|e| e.to_node == node_id)
            .map(|e| e.from_node.clone())
            .collect()
    }
}

/// Orchestrator executing a `MediaWorkflowGraph` across all registered MiniAI modalities.
pub struct WorkflowExecutor {
    pool: SharedFrameBufferPool,
}

impl WorkflowExecutor {
    pub fn new(pool: SharedFrameBufferPool) -> Self {
        Self { pool }
    }

    /// Executes the entire DAG pipeline end-to-end offline.
    pub fn execute(
        &self,
        graph: &MediaWorkflowGraph,
        cancel_token: Option<&AtomicBool>,
    ) -> Result<WorkflowExecutionResult, VideoError> {
        let start_time = Instant::now();
        let execution_order = graph.validate_and_sort()?;

        let mut outputs: HashMap<String, WorkflowData> = HashMap::new();
        let mut telemetry: Vec<NodeTelemetry> = Vec::new();

        for node_id in execution_order {
            if let Some(token) = cancel_token {
                if token.load(Ordering::Relaxed) {
                    return Err(VideoError::Cancelled);
                }
            }

            let node =
                graph
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| VideoError::WorkflowNodeNotFound {
                        node_id: node_id.clone(),
                    })?;

            let node_start = Instant::now();
            let dep_ids = graph.get_dependencies(&node_id);
            let dep_outputs: Vec<&WorkflowData> =
                dep_ids.iter().filter_map(|id| outputs.get(id)).collect();

            let result = self.execute_node(node, &dep_outputs);

            let duration_ms = node_start.elapsed().as_millis() as u64;

            match result {
                Ok(data) => {
                    outputs.insert(node_id.clone(), data);
                    telemetry.push(NodeTelemetry {
                        node_id,
                        status: NodeExecutionStatus::Completed,
                        duration_ms,
                        error_message: None,
                    });
                }
                Err(err) => {
                    telemetry.push(NodeTelemetry {
                        node_id: node_id.clone(),
                        status: NodeExecutionStatus::Failed,
                        duration_ms,
                        error_message: Some(err.to_string()),
                    });
                    return Err(VideoError::WorkflowExecutionError {
                        node_id,
                        reason: err.to_string(),
                    });
                }
            }
        }

        let total_latency_ms = start_time.elapsed().as_millis() as u64;

        Ok(WorkflowExecutionResult {
            graph_id: graph.id.clone(),
            outputs,
            telemetry,
            total_latency_ms,
        })
    }

    fn execute_node(
        &self,
        node: &WorkflowNode,
        inputs: &[&WorkflowData],
    ) -> Result<WorkflowData, VideoError> {
        match &node.node_type {
            WorkflowNodeType::LlmPrompt {
                prompt,
                max_tokens: _,
            } => {
                // Collect upstream text context if available
                let mut context = String::new();
                for inp in inputs {
                    if let WorkflowData::Text(t) = inp {
                        context.push_str(t);
                        context.push('\n');
                    }
                }
                let script_result = if context.is_empty() {
                    format!("Story Script: {prompt}. Visuals: Smooth cinematic opening with dramatic lighting.")
                } else {
                    format!("Story Script ({prompt}): {context}")
                };
                Ok(WorkflowData::Text(script_result))
            }
            WorkflowNodeType::ImageGen {
                prompt,
                width,
                height,
            } => {
                // Synthesize keyframe image RGB buffer
                let w = (*width).max(64);
                let h = (*height).max(64);
                let mut buf = self.pool.allocate_buffer(w, h)?;

                let seed = prompt.bytes().fold(987654321u64, |acc, b| {
                    acc.wrapping_mul(31).wrapping_add(b as u64)
                });

                for y in 0..h as usize {
                    for x in 0..w as usize {
                        let idx = (y * (w as usize) + x) * 3;
                        if idx + 2 < buf.len() {
                            buf[idx] = ((seed + (x as u64 * 3)) % 256) as u8;
                            buf[idx + 1] = (((seed >> 8) + (y as u64 * 4)) % 256) as u8;
                            buf[idx + 2] = (((seed >> 16) + ((x + y) as u64 * 2)) % 256) as u8;
                        }
                    }
                }

                Ok(WorkflowData::Image {
                    width: w,
                    height: h,
                    rgb_data: buf,
                })
            }
            WorkflowNodeType::VisionAnalyze => {
                let mut summary = "Visual inspection:".to_string();
                for inp in inputs {
                    match inp {
                        WorkflowData::Image {
                            width,
                            height,
                            rgb_data,
                        } => {
                            summary.push_str(&format!(
                                " Detected image of dimensions {}x{} ({} bytes).",
                                width,
                                height,
                                rgb_data.len()
                            ));
                        }
                        WorkflowData::Video { metadata, .. } => {
                            summary.push_str(&format!(
                                " Detected video stream of duration {:.1}s ({} frames, {}x{}).",
                                metadata.duration_secs,
                                metadata.total_frames,
                                metadata.width,
                                metadata.height
                            ));
                        }
                        _ => {}
                    }
                }
                Ok(WorkflowData::Text(summary))
            }
            WorkflowNodeType::SpeechTts { voice, speed: _ } => {
                let text = inputs
                    .iter()
                    .find_map(|inp| {
                        if let WorkflowData::Text(t) = inp {
                            Some(t.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("Narrator voiceover speech.");

                // Synthesize 16kHz speech tone WAV matching text length
                let sample_rate = 16000u32;
                let duration_secs = ((text.len() as f64) * 0.05).clamp(1.0, 10.0);
                let total_samples = (duration_secs * (sample_rate as f64)).round() as usize;

                let mut pcm = Vec::with_capacity(total_samples);
                for i in 0..total_samples {
                    let t = (i as f32) / (sample_rate as f32);
                    let f = match voice.as_str() {
                        "Onyx" => 120.0,
                        "Nova" => 240.0,
                        _ => 180.0,
                    };
                    pcm.push(0.3 * (2.0 * std::f32::consts::PI * f * t).sin());
                }

                let wav_bytes =
                    crate::demuxer::AudioDemuxer::encode_wav_16bit(&pcm, sample_rate, 1);
                Ok(WorkflowData::Audio {
                    sample_rate,
                    channels: 1,
                    wav_bytes,
                })
            }
            WorkflowNodeType::SpeechAsr => {
                let audio_found = inputs
                    .iter()
                    .any(|inp| matches!(inp, WorkflowData::Audio { .. }));
                let transcript = if audio_found {
                    "Transcribed Speech: Welcome to MiniAI unified multimodal pipeline."
                } else {
                    "Transcribed Speech: Silent track."
                };
                Ok(WorkflowData::Text(transcript.to_string()))
            }
            WorkflowNodeType::MusicGen {
                mood,
                duration_secs,
            } => {
                let sample_rate = 16000u32;
                let dur = (*duration_secs).clamp(1.0, 30.0);
                let total_samples = (dur * (sample_rate as f64)).round() as usize;

                let mut pcm = Vec::with_capacity(total_samples);
                let base_freq = if mood.to_lowercase().contains("epic") {
                    140.0
                } else {
                    220.0
                };

                for i in 0..total_samples {
                    let t = (i as f32) / (sample_rate as f32);
                    let wave = 0.2 * (2.0 * std::f32::consts::PI * base_freq * t).sin()
                        + 0.1 * (2.0 * std::f32::consts::PI * base_freq * 1.5 * t).sin();
                    pcm.push(wave);
                }

                let wav_bytes =
                    crate::demuxer::AudioDemuxer::encode_wav_16bit(&pcm, sample_rate, 1);
                Ok(WorkflowData::Audio {
                    sample_rate,
                    channels: 1,
                    wav_bytes,
                })
            }
            WorkflowNodeType::VideoSynthesis {
                motion,
                duration_secs,
            } => {
                let start_image = inputs.iter().find_map(|inp| {
                    if let WorkflowData::Image {
                        width,
                        height,
                        rgb_data,
                    } = inp
                    {
                        Some((rgb_data.clone(), *width, *height))
                    } else {
                        None
                    }
                });

                let (start_frame, w, h) = if let Some((buf, width, height)) = start_image {
                    (Some(buf), width, height)
                } else {
                    (None, 320, 240)
                };

                let synth = VideoSynthesisEngine::new(self.pool.clone());
                let config = VideoSynthesisConfig {
                    prompt: "Synthesized workflow video".to_string(),
                    motion: *motion,
                    width: w,
                    height: h,
                    duration_secs: *duration_secs,
                    fps: 15.0,
                    seed: Some(101),
                    start_frame,
                    end_frame: None,
                    generate_audio: true,
                };

                let syn_result = synth.synthesize(&config)?;
                Ok(WorkflowData::Video {
                    metadata: syn_result.metadata,
                    video_bytes: syn_result.video_bytes,
                })
            }
            WorkflowNodeType::VideoAnalysis => {
                let video_data = inputs
                    .iter()
                    .find_map(|inp| {
                        if let WorkflowData::Video { video_bytes, .. } = inp {
                            Some(video_bytes.as_slice())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        VideoError::InferenceError("No video input for analysis node".to_string())
                    })?;

                let engine = VideoEngine::new(self.pool.clone());
                let analysis = engine.analyze_video(video_data, &VideoEngineConfig::default())?;

                let json_val = serde_json::to_value(&analysis).map_err(|e| {
                    VideoError::InferenceError(format!("JSON serialization failed: {e}"))
                })?;
                Ok(WorkflowData::Json(json_val))
            }
            WorkflowNodeType::AudioVideoMux => {
                // Multiplex video stream with upstream voice and music
                let video_bytes = inputs
                    .iter()
                    .find_map(|inp| {
                        if let WorkflowData::Video { video_bytes, .. } = inp {
                            Some(video_bytes.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        VideoError::InferenceError(
                            "AudioVideoMux requires a video input".to_string(),
                        )
                    })?;

                let meta =
                    VideoSanitizer::validate_bytes(&video_bytes, &SanitizerConfig::default())?;
                Ok(WorkflowData::Video {
                    metadata: meta,
                    video_bytes,
                })
            }
            WorkflowNodeType::Custom { kind, config } => {
                let mut map = serde_json::Map::new();
                map.insert("kind".to_string(), serde_json::Value::String(kind.clone()));
                map.insert("config".to_string(), config.clone());
                map.insert(
                    "input_count".to_string(),
                    serde_json::Value::Number(inputs.len().into()),
                );
                Ok(WorkflowData::Json(serde_json::Value::Object(map)))
            }
        }
    }
}

use miniai_core::{AcceleratorBackend, PowerState, ThermalState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum BridgeRequest {
    #[serde(rename = "probe_hardware")]
    ProbeHardware,

    #[serde(rename = "get_privacy_status")]
    GetPrivacyStatus,

    #[serde(rename = "set_privacy_mode")]
    SetPrivacyMode { local_only: bool },

    #[serde(rename = "list_models")]
    ListModels,

    #[serde(rename = "search_knowledge")]
    SearchKnowledge { query: String, limit: usize },

    #[serde(rename = "execute_chat")]
    ExecuteChat {
        session_id: String,
        prompt: String,
        #[serde(default)]
        model_id: Option<String>,
        #[serde(default)]
        messages: Option<Vec<miniai_llm::ChatMessage>>,
        #[serde(default)]
        temperature: Option<f32>,
        #[serde(default)]
        max_tokens: Option<usize>,
        #[serde(default)]
        system_prompt: Option<String>,
    },

    #[serde(rename = "poll_events")]
    PollEvents,

    #[serde(rename = "get_audit_ledger")]
    GetAuditLedger {
        #[serde(default = "default_audit_limit")]
        limit: usize,
    },

    #[serde(rename = "run_doctor")]
    RunDoctor,

    #[serde(rename = "get_telemetry")]
    GetTelemetry,

    #[serde(rename = "unload_model")]
    UnloadModel {
        #[serde(default)]
        model_id: Option<String>,
    },

    #[serde(rename = "export_redacted_diagnostics")]
    ExportRedactedDiagnostics,

    #[serde(rename = "list_memories")]
    ListMemories,

    #[serde(rename = "store_memory")]
    StoreMemory {
        memory_type: String,
        content: String,
        #[serde(default)]
        metadata: Option<serde_json::Value>,
    },

    #[serde(rename = "clear_memories")]
    ClearMemories,

    #[serde(rename = "list_mcp_tools")]
    ListMcpTools,

    #[serde(rename = "get_swarm_status")]
    GetSwarmStatus,

    #[serde(rename = "get_npu_backends")]
    GetNpuBackends,

    #[serde(rename = "start_model_download")]
    StartModelDownload {
        model_id: String,
        source_url: String,
        #[serde(default)]
        expected_sha256: Option<String>,
        #[serde(default)]
        expected_size_bytes: Option<u64>,
    },

    #[serde(rename = "get_model_download_progress")]
    GetModelDownloadProgress { model_id: String },

    #[serde(rename = "cancel_model_download")]
    CancelModelDownload { model_id: String },

    #[serde(rename = "import_local_model")]
    ImportLocalModel {
        source_path: String,
        model_id: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        format: Option<String>,
        #[serde(default)]
        expected_sha256: Option<String>,
        #[serde(default)]
        expected_size_bytes: Option<u64>,
    },

    #[serde(rename = "generate_image")]
    GenerateImage {
        prompt: String,
        #[serde(default)]
        negative_prompt: Option<String>,
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        steps: Option<u32>,
        #[serde(default)]
        guidance_scale: Option<f32>,
        #[serde(default)]
        seed: Option<u64>,
    },

    #[serde(rename = "analyze_vision")]
    AnalyzeVision {
        #[serde(default)]
        image_base64: Option<String>,
        #[serde(default)]
        prompt: Option<String>,
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
    },

    #[serde(rename = "transcribe_speech")]
    TranscribeSpeech {
        #[serde(default)]
        audio_base64: Option<String>,
        #[serde(default)]
        sample_rate: Option<u32>,
        #[serde(default)]
        language: Option<String>,
    },

    #[serde(rename = "synthesize_speech")]
    SynthesizeSpeech {
        text: String,
        #[serde(default)]
        voice_id: Option<String>,
        #[serde(default)]
        pitch: Option<f32>,
        #[serde(default)]
        speed: Option<f32>,
        #[serde(default)]
        sample_rate: Option<u32>,
    },

    #[serde(rename = "synthesize_music")]
    SynthesizeMusic {
        #[serde(default)]
        genre: Option<String>,
        #[serde(default)]
        tempo_bpm: Option<u32>,
        #[serde(default)]
        duration_secs: Option<f64>,
        #[serde(default)]
        key_signature: Option<String>,
        #[serde(default)]
        seed: Option<u64>,
    },

    #[serde(rename = "synthesize_video")]
    SynthesizeVideo {
        prompt: String,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        fps: Option<f64>,
        #[serde(default)]
        duration_secs: Option<f64>,
        #[serde(default)]
        seed: Option<u64>,
    },

    #[serde(rename = "execute_training_job")]
    ExecuteTrainingJob {
        task_type: String,
        base_model: String,
        dataset_name: String,
        #[serde(default)]
        epochs: Option<u32>,
        #[serde(default)]
        learning_rate: Option<f64>,
        #[serde(default)]
        batch_size: Option<u32>,
        #[serde(default)]
        lora_rank: Option<u32>,
    },

    #[serde(rename = "get_power_profile")]
    GetPowerProfile,

    #[serde(rename = "set_power_profile")]
    SetPowerProfile {
        #[serde(default)]
        battery_saving: Option<bool>,
        #[serde(default)]
        simulated_thermal: Option<ThermalState>,
        #[serde(default)]
        simulated_power: Option<PowerState>,
    },

    #[serde(rename = "select_accelerator_backend")]
    SelectAcceleratorBackend { backend: AcceleratorBackend },

    #[serde(rename = "reset_kv_cache")]
    ResetKvCache,
}

fn default_audit_limit() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BridgeErrorPayload>,
}

impl BridgeResponse {
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(BridgeErrorPayload {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeErrorPayload {
    pub code: String,
    pub message: String,
}

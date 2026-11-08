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

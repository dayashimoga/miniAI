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
    },

    #[serde(rename = "poll_events")]
    PollEvents,

    #[serde(rename = "get_audit_ledger")]
    GetAuditLedger {
        #[serde(default = "default_audit_limit")]
        limit: usize,
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

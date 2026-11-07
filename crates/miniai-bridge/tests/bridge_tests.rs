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

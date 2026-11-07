use miniai_cloud::{ConsentAction, ConsentAuditRecord, ConsentLedger};
use miniai_core::Modality;
use miniai_storage::StorageEngine;
use std::sync::Arc;

fn create_test_ledger() -> ConsentLedger {
    let storage = Arc::new(StorageEngine::in_memory().expect("in-memory db"));
    ConsentLedger::new(storage)
}

#[test]
fn test_ledger_record_and_list_events() {
    let ledger = create_test_ledger();
    assert_eq!(ledger.count_events().unwrap(), 0);

    let rec1 = ConsentAuditRecord {
        id: None,
        token_id: "tok_1".to_string(),
        action: ConsentAction::Granted,
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        payload_bytes: 0,
        epoch_secs: 1000,
        details_json: Some(r#"{"test":true}"#.to_string()),
    };

    let id1 = ledger.record_event(&rec1).unwrap();
    assert!(id1 > 0);
    assert_eq!(ledger.count_events().unwrap(), 1);

    let rec2 = ConsentAuditRecord {
        id: None,
        token_id: "tok_1".to_string(),
        action: ConsentAction::Verified,
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        payload_bytes: 1024,
        epoch_secs: 1010,
        details_json: None,
    };
    ledger.record_event(&rec2).unwrap();
    assert_eq!(ledger.count_events().unwrap(), 2);

    let events = ledger.list_events(10).unwrap();
    assert_eq!(events.len(), 2);
    // Ordered descending by id
    assert_eq!(events[0].action, ConsentAction::Verified);
    assert_eq!(events[1].action, ConsentAction::Granted);
}

#[test]
fn test_ledger_query_filters() {
    let ledger = create_test_ledger();

    let items = [
        ("tok_a", ConsentAction::Granted, Modality::Llm, 1000),
        ("tok_a", ConsentAction::Verified, Modality::Llm, 1100),
        ("tok_b", ConsentAction::Granted, Modality::Vision, 1200),
        ("tok_a", ConsentAction::Revoked, Modality::Llm, 1300),
        (
            "tok_b",
            ConsentAction::PayloadDispatched,
            Modality::Vision,
            1400,
        ),
        ("tok_c", ConsentAction::Rejected, Modality::Music, 1500),
    ];

    for (tid, act, modl, ep) in items {
        ledger
            .record_event(&ConsentAuditRecord {
                id: None,
                token_id: tid.to_string(),
                action: act,
                modality: modl,
                target_endpoint: "https://cloud.miniai.internal".to_string(),
                payload_bytes: 500,
                epoch_secs: ep,
                details_json: None,
            })
            .unwrap();
    }

    assert_eq!(ledger.count_events().unwrap(), 6);

    // Filter by token
    let tok_a_events = ledger.query_by_token("tok_a").unwrap();
    assert_eq!(tok_a_events.len(), 3);
    assert!(tok_a_events.iter().all(|e| e.token_id == "tok_a"));

    let tok_b_events = ledger.query_by_token("tok_b").unwrap();
    assert_eq!(tok_b_events.len(), 2);

    // Filter by action
    let granted = ledger.query_by_action(ConsentAction::Granted).unwrap();
    assert_eq!(granted.len(), 2);

    let rejected = ledger.query_by_action(ConsentAction::Rejected).unwrap();
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].token_id, "tok_c");

    // Filter by modality
    let llm_events = ledger.query_by_modality(Modality::Llm).unwrap();
    assert_eq!(llm_events.len(), 3);

    let vision_events = ledger.query_by_modality(Modality::Vision).unwrap();
    assert_eq!(vision_events.len(), 2);

    // Filter by time range
    let window = ledger.query_by_time_range(1100, 1300).unwrap();
    assert_eq!(window.len(), 3);
    assert!(window
        .iter()
        .all(|e| e.epoch_secs >= 1100 && e.epoch_secs <= 1300));
}

#[test]
fn test_ledger_append_only_immutability() {
    let ledger = create_test_ledger();

    let mut ids = Vec::new();
    for i in 1..=5 {
        let id = ledger
            .record_event(&ConsentAuditRecord {
                id: None,
                token_id: format!("tok_{i}"),
                action: ConsentAction::Granted,
                modality: Modality::Video,
                target_endpoint: "https://cloud.miniai.internal".to_string(),
                payload_bytes: i * 100,
                epoch_secs: 1000 + i,
                details_json: None,
            })
            .unwrap();
        ids.push(id);
    }

    // Monotonically increasing IDs
    for w in ids.windows(2) {
        assert!(w[0] < w[1]);
    }

    assert_eq!(ledger.count_events().unwrap(), 5);
}

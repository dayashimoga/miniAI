use miniai_memory::{
    EpisodicMemoryService, SemanticMemoryService, ThreeTierMemoryManager, TurnRole, WorkingMemory,
};
use miniai_storage::{ChatMessageRecord, MessageRole, StorageEngine};
use std::sync::Arc;

#[test]
fn test_working_memory_budgeting_and_pinned_preservation() {
    // Budget of ~50 tokens (~200 chars)
    let mut working = WorkingMemory::new(50);

    working.set_pinned_system_prompt("System: You are MiniAI, a local assistant.");
    assert_eq!(working.active_turns().len(), 1);
    assert!(working.active_turns()[0].is_pinned);

    // Add unpinned turn 1
    working.add_turn(TurnRole::User, "Hello! Can you help me?", false);
    // Add unpinned turn 2
    working.add_turn(TurnRole::Assistant, "Yes, I am ready to help.", false);
    assert_eq!(working.active_turns().len(), 3);

    // Add a large unpinned message that forces eviction of older turns
    let large_text = "A".repeat(180);
    working.add_turn(TurnRole::User, &large_text, false);

    let turns = working.active_turns();
    // Pinned system prompt must still be present!
    assert_eq!(turns[0].role, TurnRole::System);
    assert!(turns[0].is_pinned);

    // Total tokens must respect the budget
    assert!(working.total_tokens() <= 50);

    // Scratchpad test
    working.add_scratchpad("calculator", "Output: 42");
    assert_eq!(working.scratchpad().len(), 1);
    assert!(working.scratchpad()[0].contains("42"));

    working.clear_scratchpad();
    assert!(working.scratchpad().is_empty());
}

#[test]
fn test_episodic_memory_session_summarization_and_search() {
    let storage = Arc::new(StorageEngine::in_memory().unwrap());
    let service = EpisodicMemoryService::new(storage);

    let messages = vec![
        ChatMessageRecord {
            id: "m1".to_string(),
            session_id: "sess-01".to_string(),
            parent_id: None,
            role: MessageRole::User,
            content: "How does the memory architecture work?".to_string(),
            token_count: 8,
            model_id: None,
            created_at_epoch_secs: 1000,
            updated_at_epoch_secs: 1000,
            is_pinned: false,
        },
        ChatMessageRecord {
            id: "m2".to_string(),
            session_id: "sess-01".to_string(),
            parent_id: Some("m1".to_string()),
            role: MessageRole::Assistant,
            content: "MiniAI uses a three-tier model: working, episodic, and semantic.".to_string(),
            token_count: 14,
            model_id: Some("test-model".to_string()),
            created_at_epoch_secs: 1005,
            updated_at_epoch_secs: 1005,
            is_pinned: false,
        },
    ];

    let record = service
        .summarize_and_store_session("sess-01", "Memory Architecture Discussion", &messages, 1010)
        .unwrap();

    assert_eq!(record.topic, "Memory Architecture Discussion");
    assert!(record.summary.contains("1 user queries"));
    assert!(record.key_decisions.contains("three-tier model"));

    let recent = service.list_recent(5).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].session_id, "sess-01");

    let search_hits = service.search("Architecture").unwrap();
    assert_eq!(search_hits.len(), 1);

    let no_hits = service.search("blockchain").unwrap();
    assert!(no_hits.is_empty());
}

#[test]
fn test_semantic_memory_facts_and_privacy_forget() {
    let storage = Arc::new(StorageEngine::in_memory().unwrap());
    let service = SemanticMemoryService::new(storage);

    // 1. Store preferences and facts
    let fact1 = service
        .store_fact(
            "preference",
            "User prefers concise Rust code with zero unsafe blocks.",
            0.98,
            1000,
        )
        .unwrap();

    let fact2 = service
        .store_fact(
            "project_fact",
            "MiniAI operates strictly in Local-Only Privacy Mode by default.",
            1.0,
            1005,
        )
        .unwrap();

    let all_facts = service.list_all().unwrap();
    assert_eq!(all_facts.len(), 2);

    // 2. Search by category
    let prefs = service.search(Some("preference"), None).unwrap();
    assert_eq!(prefs.len(), 1);
    assert_eq!(prefs[0].id, fact1.id);

    // 3. Search by query
    let privacy_hits = service.search(None, Some("Privacy Mode")).unwrap();
    assert_eq!(privacy_hits.len(), 1);
    assert_eq!(privacy_hits[0].id, fact2.id);

    // 4. Privacy "Forget Memory" guarantee
    let forgotten = service.forget(&fact1.id).unwrap();
    assert!(forgotten);

    let remaining = service.list_all().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, fact2.id);
}

#[test]
fn test_three_tier_memory_manager_assembly() {
    let storage = Arc::new(StorageEngine::in_memory().unwrap());
    let manager = ThreeTierMemoryManager::new(storage, 1000);

    // Add working turns
    manager
        .record_turn(TurnRole::User, "Hello MiniAI!", false)
        .unwrap();
    assert_eq!(manager.active_working_turns().unwrap().len(), 1);

    // Add semantic fact
    manager
        .semantic_service()
        .store_fact(
            "preference",
            "User prefers Markdown formatting.",
            0.95,
            2000,
        )
        .unwrap();

    // Assemble context
    let context = manager.assemble_memory_context("formatting").unwrap();
    assert!(context.contains("### Known User Facts & Preferences:"));
    assert!(context.contains("User prefers Markdown formatting."));
}

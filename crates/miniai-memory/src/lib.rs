#![forbid(unsafe_code)]

//! # MiniAI Multi-Tier Memory Crate (`miniai-memory`)
//!
//! Implements the three-tier memory architecture for MiniAI:
//! 1. **Working Memory**: Token-budgeted rolling context window with pinned instructions and scratchpad.
//! 2. **Episodic Memory**: Cross-session conversation summarization and indexing in SQLite.
//! 3. **Semantic Fact Memory**: Durable user preferences and project knowledge with privacy erasure.

pub mod episodic;
pub mod error;
pub mod manager;
pub mod semantic;
pub mod working;

pub use episodic::EpisodicMemoryService;
pub use error::MemoryError;
pub use manager::ThreeTierMemoryManager;
pub use semantic::SemanticMemoryService;
pub use working::{TurnRole, WorkingMemory, WorkingTurn};

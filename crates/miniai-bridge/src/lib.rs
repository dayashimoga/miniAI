//! # MiniAI Native Bridge (`miniai-bridge`)
//!
//! Strongly-typed cross-platform FFI bridge and dispatch engine connecting Flutter
//! (Windows, Linux, macOS, Android, iOS) and local IPC clients directly to MiniAI Rust core crates.

pub mod contracts;
pub mod engine;
pub mod error;
pub mod ffi;

pub use contracts::{BridgeErrorPayload, BridgeRequest, BridgeResponse};
pub use engine::BridgeEngine;
pub use error::BridgeError;
pub use ffi::{
    miniai_bridge_dispatch, miniai_bridge_free, miniai_bridge_init, miniai_bridge_string_free,
};

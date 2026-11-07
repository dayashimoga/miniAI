use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

use crate::engine::BridgeEngine;

/// Initializes the MiniAI Native Bridge Engine.
///
/// If `db_path` is NULL or empty, initializes with in-memory SQLite storage.
/// Returns a raw pointer to the heap-allocated `BridgeEngine`, or NULL on failure.
///
/// # Safety
/// If `db_path` is non-null, it must point to a valid, null-terminated C string containing
/// a valid UTF-8 filesystem path. The returned pointer must be deallocated using [`miniai_bridge_free`].
#[no_mangle]
pub unsafe extern "C" fn miniai_bridge_init(db_path: *const c_char) -> *mut BridgeEngine {
    let engine_res = if db_path.is_null() {
        BridgeEngine::new_in_memory()
    } else {
        match CStr::from_ptr(db_path).to_str() {
            Ok(path_str) if !path_str.trim().is_empty() => BridgeEngine::open(Path::new(path_str)),
            _ => BridgeEngine::new_in_memory(),
        }
    };

    match engine_res {
        Ok(engine) => Box::into_raw(Box::new(engine)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a `BridgeEngine` previously initialized with `miniai_bridge_init`.
///
/// # Safety
/// `engine` must either be null or point to a valid, live heap-allocated `BridgeEngine`
/// returned from [`miniai_bridge_init`]. It must not be used after being freed.
#[no_mangle]
pub unsafe extern "C" fn miniai_bridge_free(engine: *mut BridgeEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// Dispatches a JSON request string to the engine and returns a heap-allocated JSON response string.
///
/// The caller is responsible for freeing the returned string using `miniai_bridge_string_free`.
///
/// # Safety
/// `engine` must point to a valid, live `BridgeEngine`.
/// `request_json` must point to a valid, null-terminated C string containing UTF-8 JSON.
/// The returned string pointer must be freed using [`miniai_bridge_string_free`].
#[no_mangle]
pub unsafe extern "C" fn miniai_bridge_dispatch(
    engine: *mut BridgeEngine,
    request_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() || request_json.is_null() {
        let err_json = r#"{"success":false,"error":{"code":"NULL_POINTER","message":"Null pointer provided to bridge dispatch"}}"#;
        return CString::new(err_json).unwrap().into_raw();
    }

    let c_str = match CStr::from_ptr(request_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            let err_json = r#"{"success":false,"error":{"code":"INVALID_UTF8","message":"Request string contains invalid UTF-8"}}"#;
            return CString::new(err_json).unwrap().into_raw();
        }
    };

    let engine_ref = &*engine;
    let response_json = engine_ref.dispatch_json(c_str);

    match CString::new(response_json) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => {
            let err_json = r#"{"success":false,"error":{"code":"NUL_BYTE_ERROR","message":"Response string contains internal NUL byte"}}"#;
            CString::new(err_json).unwrap().into_raw()
        }
    }
}

/// Frees a string returned by `miniai_bridge_dispatch`.
///
/// # Safety
/// `ptr` must either be null or point to a valid, live C-compatible string allocated by
/// [`miniai_bridge_dispatch`]. It must not be accessed after being freed.
#[no_mangle]
pub unsafe extern "C" fn miniai_bridge_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

//! C-ABI FFI Exports
//!
//! These functions are exported for JUCE (C++) to call.

use crate::{init_bridge, get_bridge, Command, StateSnapshot};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Initialize the bridge. Call once at startup from JUCE.
#[no_mangle]
pub extern "C" fn rysyn_init() -> bool {
    init_bridge();
    true
}

/// Get state snapshot as JSON string. Caller must free with rysyn_free_string.
#[no_mangle]
pub extern "C" fn rysyn_get_state_json() -> *mut c_char {
    let bridge = match get_bridge() {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };
    
    let state = bridge.get_state();
    match serde_json::to_string(&state) {
        Ok(json) => {
            match CString::new(json) {
                Ok(cstr) => cstr.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Update state from JSON. Called by JUCE audio core.
#[no_mangle]
pub extern "C" fn rysyn_update_state_json(json: *const c_char) -> bool {
    if json.is_null() {
        return false;
    }
    
    let bridge = match get_bridge() {
        Some(b) => b,
        None => return false,
    };
    
    let c_str = unsafe { CStr::from_ptr(json) };
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    match serde_json::from_str::<StateSnapshot>(json_str) {
        Ok(state) => {
            bridge.update_state(state);
            true
        }
        Err(_) => false,
    }
}

/// Send a command as JSON. Called by UI or scripts.
#[no_mangle]
pub extern "C" fn rysyn_send_command_json(json: *const c_char) -> bool {
    if json.is_null() {
        return false;
    }
    
    let bridge = match get_bridge() {
        Some(b) => b,
        None => return false,
    };
    
    let c_str = unsafe { CStr::from_ptr(json) };
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    match serde_json::from_str::<Command>(json_str) {
        Ok(cmd) => bridge.send_command(cmd).is_ok(),
        Err(_) => false,
    }
}

/// Try to receive next command as JSON. Returns null if queue is empty.
/// Caller must free with rysyn_free_string.
#[no_mangle]
pub extern "C" fn rysyn_recv_command_json() -> *mut c_char {
    let bridge = match get_bridge() {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };
    
    match bridge.try_recv_command() {
        Some(cmd) => {
            match serde_json::to_string(&cmd) {
                Ok(json) => {
                    match CString::new(json) {
                        Ok(cstr) => cstr.into_raw(),
                        Err(_) => std::ptr::null_mut(),
                    }
                }
                Err(_) => std::ptr::null_mut(),
            }
        }
        None => std::ptr::null_mut(),
    }
}

/// Free a string allocated by Rust
#[no_mangle]
pub extern "C" fn rysyn_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// === Transport shortcuts (avoid JSON overhead for hot paths) ===

#[no_mangle]
pub extern "C" fn rysyn_transport_play() -> bool {
    get_bridge()
        .map(|b| b.send_command(Command::Play).is_ok())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn rysyn_transport_pause() -> bool {
    get_bridge()
        .map(|b| b.send_command(Command::Pause).is_ok())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn rysyn_transport_stop() -> bool {
    get_bridge()
        .map(|b| b.send_command(Command::Stop).is_ok())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn rysyn_transport_set_bpm(bpm: f64) -> bool {
    get_bridge()
        .map(|b| b.send_command(Command::SetBpm { bpm }).is_ok())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn rysyn_transport_set_playhead(beats: f64) -> bool {
    get_bridge()
        .map(|b| b.send_command(Command::SetPlayhead { beats }).is_ok())
        .unwrap_or(false)
}

/// Get current playhead position in beats (fast path, no JSON)
#[no_mangle]
pub extern "C" fn rysyn_get_playhead_beats() -> f64 {
    get_bridge()
        .map(|b| b.get_state().transport.playhead_beats)
        .unwrap_or(0.0)
}

/// Get current BPM (fast path)
#[no_mangle]
pub extern "C" fn rysyn_get_bpm() -> f64 {
    get_bridge()
        .map(|b| b.get_state().transport.bpm)
        .unwrap_or(120.0)
}

/// Check if playing (fast path)
#[no_mangle]
pub extern "C" fn rysyn_is_playing() -> bool {
    get_bridge()
        .map(|b| b.get_state().transport.is_playing)
        .unwrap_or(false)
}

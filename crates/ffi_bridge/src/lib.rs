//! FFI Bridge Layer
//!
//! This crate provides the C-ABI interface between:
//! - JUCE Audio Core (C++) ↔ Rust Engine
//! - egui UI (Rust) ↔ JUCE Audio Core
//!
//! Data Flow:
//! 1. UI reads StateSnapshot (read-only) each frame
//! 2. UI sends Commands to CommandQueue (write)
//! 3. JUCE consumes commands on message thread
//! 4. JUCE updates state and produces new snapshot

pub mod commands;
pub mod state;
pub mod ffi;

pub use commands::*;
pub use state::*;

use parking_lot::RwLock;
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::Arc;

/// Global bridge instance for FFI access
static mut BRIDGE: Option<Bridge> = None;

/// The main bridge connecting UI and Audio Core
pub struct Bridge {
    /// Current state snapshot (read by UI)
    pub state: Arc<RwLock<StateSnapshot>>,
    /// Command sender (UI -> Audio Core)
    pub command_tx: Sender<Command>,
    /// Command receiver (consumed by Audio Core)
    pub command_rx: Receiver<Command>,
}

impl Bridge {
    pub fn new() -> Self {
        let (command_tx, command_rx) = bounded(1024);
        Self {
            state: Arc::new(RwLock::new(StateSnapshot::default())),
            command_tx,
            command_rx,
        }
    }

    /// Send a command from UI to Audio Core
    pub fn send_command(&self, cmd: Command) -> Result<(), String> {
        self.command_tx.try_send(cmd).map_err(|e| e.to_string())
    }

    /// Get current state snapshot (for UI)
    pub fn get_state(&self) -> StateSnapshot {
        self.state.read().clone()
    }

    /// Update state (called by Audio Core)
    pub fn update_state(&self, new_state: StateSnapshot) {
        *self.state.write() = new_state;
    }

    /// Try receive a command (called by Audio Core)
    pub fn try_recv_command(&self) -> Option<Command> {
        self.command_rx.try_recv().ok()
    }
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the global bridge (call once at startup)
pub fn init_bridge() -> &'static Bridge {
    unsafe {
        if BRIDGE.is_none() {
            BRIDGE = Some(Bridge::new());
        }
        BRIDGE.as_ref().unwrap()
    }
}

/// Get the global bridge reference
pub fn get_bridge() -> Option<&'static Bridge> {
    unsafe { BRIDGE.as_ref() }
}

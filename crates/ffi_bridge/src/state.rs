//! State Snapshot
//!
//! Read-only state that UI reads each frame.
//! This is a "frozen" view of the audio engine state.

use serde::{Deserialize, Serialize};

/// Transport state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportState {
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub playhead_beats: f64,
    pub playhead_seconds: f64,
    pub tempo: f64,
    pub time_sig_num: u32,
    pub time_sig_denom: u32,
    pub loop_start_beats: f64,
    pub loop_end_beats: f64,
}

/// Audio meter levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeterLevels {
    pub master_l: f32,
    pub master_r: f32,
    pub track_levels_l: Vec<f32>,
    pub track_levels_r: Vec<f32>,
}

/// Track state (read-only view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub id: u32,
    pub name: String,
    pub color: u32, // RGBA packed
    pub volume: f32,
    pub pan: f32,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub is_armed: bool,
    pub is_midi: bool,
    pub is_selected: bool,
    pub input_name: String,
    pub output_name: String,
    pub plugins: Vec<Option<String>>, // Plugin names, None for empty slots
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            color: 0xFF808080,
            volume: 1.0,
            pan: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
            is_midi: false,
            is_selected: false,
            input_name: "No Input".to_string(),
            output_name: "Master".to_string(),
            plugins: vec![None; 8], // 8 plugin slots
        }
    }
}

/// Available plugin info (for browser)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub format: String, // "VST3", "AU", "CLAP"
    pub path: String,
    pub is_instrument: bool,
}

/// Plugin slot state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSlotState {
    pub id: u32,
    pub name: String,
    pub vendor: String,
    pub is_bypassed: bool,
    pub is_loaded: bool,
    pub has_editor: bool,
}

/// Clip/Item state on timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipState {
    pub id: u32,
    pub track_id: u32,
    pub start_beats: f64,
    pub length_beats: f64,
    pub offset_beats: f64,
    pub name: String,
    pub color: u32,
    pub is_midi: bool,
    pub is_selected: bool,
}

impl Default for ClipState {
    fn default() -> Self {
        Self {
            id: 0,
            track_id: 0,
            start_beats: 0.0,
            length_beats: 4.0,
            offset_beats: 0.0,
            name: "Clip".to_string(),
            color: 0xFF6496FF,
            is_midi: false,
            is_selected: false,
        }
    }
}

/// Complete state snapshot for UI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub transport: TransportState,
    pub meters: MeterLevels,
    pub tracks: Vec<TrackState>,
    pub clips: Vec<ClipState>,
    pub available_plugins: Vec<PluginInfo>,
    pub master_volume: f32,
    pub cpu_load: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub audio_device_name: String,
    pub project_name: String,
    pub is_modified: bool,
}

impl StateSnapshot {
    pub fn new() -> Self {
        Self {
            transport: TransportState {
                tempo: 120.0,
                time_sig_num: 4,
                time_sig_denom: 4,
                loop_end_beats: 8.0,
                ..Default::default()
            },
            master_volume: 1.0,
            sample_rate: 44100,
            buffer_size: 512,
            audio_device_name: "Default".to_string(),
            project_name: "Untitled".to_string(),
            ..Default::default()
        }
    }
}

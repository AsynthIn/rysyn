//! State Snapshot
//!
//! Read-only state that UI reads each frame.
//! This is a "frozen" view of the audio engine state.

use serde::{Deserialize, Serialize};

/// Transport state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct TransportState {
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub playhead_beats: f64,
    pub playhead_seconds: f64,
    pub bpm: f64,
    pub time_sig_num: u8,
    pub time_sig_denom: u8,
    pub loop_start_beats: f64,
    pub loop_end_beats: f64,
}

/// Audio meter levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct MeterLevels {
    pub peak_l: f32,
    pub peak_r: f32,
    pub rms_l: f32,
    pub rms_r: f32,
}

/// Track state (read-only view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub id: u32,
    pub name: String,
    pub color: u32, // RGBA packed
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub armed: bool,
    pub meters: MeterLevels,
    pub clip_count: u32,
    pub plugin_count: u32,
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
    pub name: String,
    pub color: u32,
    pub is_midi: bool,
    pub is_selected: bool,
}

/// Complete state snapshot for UI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub transport: TransportState,
    pub master_meters: MeterLevels,
    pub tracks: Vec<TrackState>,
    pub clips: Vec<ClipState>,
    pub selected_track_id: Option<u32>,
    pub selected_clip_ids: Vec<u32>,
    pub cpu_load: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub project_name: String,
    pub is_modified: bool,
}

impl StateSnapshot {
    pub fn new() -> Self {
        Self {
            transport: TransportState {
                bpm: 120.0,
                time_sig_num: 4,
                time_sig_denom: 4,
                ..Default::default()
            },
            sample_rate: 44100,
            buffer_size: 512,
            project_name: "Untitled".to_string(),
            ..Default::default()
        }
    }
}

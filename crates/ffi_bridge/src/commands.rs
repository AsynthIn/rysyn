//! Command Queue
//!
//! Commands sent from UI to Audio Core.
//! These are processed on the JUCE message thread (non-realtime safe).

use serde::{Deserialize, Serialize};

/// All possible commands from UI to Audio Core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // === Transport ===
    Play,
    Pause,
    Stop,
    ToggleRecord,
    ToggleLoop,
    SetPlayhead { beats: f64 },
    SetBpm { bpm: f64 },
    SetTimeSignature { num: u8, denom: u8 },
    SetLoopRegion { start_beats: f64, end_beats: f64 },

    // === Project ===
    NewProject,
    LoadProject { path: String },
    SaveProject { path: String },

    // === Tracks ===
    AddTrack { track_type: TrackType },
    RemoveTrack { track_id: u32 },
    RenameTrack { track_id: u32, name: String },
    SetTrackVolume { track_id: u32, volume: f32 },
    SetTrackPan { track_id: u32, pan: f32 },
    SetTrackMute { track_id: u32, mute: bool },
    SetTrackSolo { track_id: u32, solo: bool },
    SetTrackArmed { track_id: u32, armed: bool },
    SetTrackColor { track_id: u32, color: u32 },
    MoveTrack { track_id: u32, new_index: u32 },

    // === Clips ===
    AddAudioClip { track_id: u32, path: String, start_beats: f64 },
    AddMidiClip { track_id: u32, start_beats: f64, length_beats: f64 },
    RemoveClip { clip_id: u32 },
    MoveClip { clip_id: u32, new_track_id: u32, new_start_beats: f64 },
    ResizeClip { clip_id: u32, new_length_beats: f64 },
    SplitClip { clip_id: u32, split_beats: f64 },
    DuplicateClip { clip_id: u32 },
    SelectClip { clip_id: u32, add_to_selection: bool },
    DeselectAllClips,

    // === MIDI Editing ===
    AddMidiNote { clip_id: u32, note: u8, velocity: u8, start_beats: f64, length_beats: f64 },
    RemoveMidiNote { clip_id: u32, note_id: u32 },
    MoveMidiNote { clip_id: u32, note_id: u32, new_note: u8, new_start_beats: f64 },

    // === Plugins (VST3) ===
    ScanPlugins,
    LoadPlugin { track_id: u32, slot: u32, plugin_id: String },
    UnloadPlugin { track_id: u32, slot: u32 },
    BypassPlugin { track_id: u32, slot: u32, bypass: bool },
    OpenPluginEditor { track_id: u32, slot: u32 },
    ClosePluginEditor { track_id: u32, slot: u32 },
    SetPluginParameter { track_id: u32, slot: u32, param_id: u32, value: f32 },

    // === Audio Settings ===
    SetAudioDevice { device_name: String },
    SetSampleRate { rate: u32 },
    SetBufferSize { size: u32 },

    // === Scripting ===
    RunScript { script: String },
    LoadScriptPlugin { path: String },

    // === Misc ===
    Undo,
    Redo,
}

/// Track type for creation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Midi,
    Instrument,
    Bus,
    Master,
}

impl Default for TrackType {
    fn default() -> Self {
        Self::Audio
    }
}

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
    SetTempo { bpm: f64 },
    SetTimeSignature { numerator: u32, denominator: u32 },
    SetLoopRegion { start_beats: f64, end_beats: f64 },

    // === Project ===
    NewProject,
    LoadProject { path: String },
    SaveProject { path: Option<String> },
    SetProjectName { name: String },

    // === Master ===
    SetMasterVolume { volume: f32 },

    // === Tracks ===
    CreateTrack { name: String, is_midi: bool },
    DeleteTrack { track_id: u32 },
    DuplicateTrack { track_id: u32 },
    RenameTrack { track_id: u32, name: String },
    SetTrackVolume { track_id: u32, volume: f32 },
    SetTrackPan { track_id: u32, pan: f32 },
    SetTrackMute { track_id: u32, muted: bool },
    SetTrackSolo { track_id: u32, soloed: bool },
    SetTrackArm { track_id: u32, armed: bool },
    SetTrackColor { track_id: u32, color: u32 },
    MoveTrack { track_id: u32, new_index: u32 },
    SelectTrack { track_id: u32, exclusive: bool },

    // === Clips ===
    ImportAudioFile { path: String },
    CreateAudioClip { track_id: u32, path: String, start_beats: f64 },
    CreateMidiClip { track_id: u32, start_beats: f64, length_beats: f64 },
    DeleteClip { clip_id: u32 },
    MoveClip { clip_id: u32, start_beats: f64, track_id: Option<u32> },
    ResizeClip { clip_id: u32, length_beats: f64 },
    RenameClip { clip_id: u32, name: String },
    SetClipColor { clip_id: u32, color: u32 },
    SplitClip { clip_id: u32, split_beats: f64 },
    DuplicateClip { clip_id: u32 },
    SelectClip { clip_id: u32, add_to_selection: bool },
    DeselectAllClips,

    // === MIDI Editing ===
    AddMidiNote { clip_id: u32, note: u8, velocity: u8, start_beats: f64, length_beats: f64 },
    DeleteMidiNote { clip_id: u32, note_id: u32 },
    MoveMidiNote { clip_id: u32, note_id: u32, new_note: u8, new_start_beats: f64 },

    // === Plugins (VST3) ===
    RescanPlugins,
    LoadPlugin { track_id: u32, plugin_id: String, slot: Option<u32> },
    RemovePlugin { track_id: u32, slot: u32 },
    BypassPlugin { track_id: u32, slot: u32, bypass: bool },
    OpenPluginEditor { track_id: u32, slot: u32 },
    ClosePluginEditor { track_id: u32, slot: u32 },
    SetPluginParameter { track_id: u32, slot: u32, param_id: u32, value: f32 },

    // === Audio Settings ===
    OpenAudioSettings,
    SetAudioDevice { device_name: String },
    SetSampleRate { rate: u32 },
    SetBufferSize { size: u32 },

    // === Scripting ===
    RunScript { script: String },
    LoadScriptPlugin { path: String },

    // === Edit ===
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
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

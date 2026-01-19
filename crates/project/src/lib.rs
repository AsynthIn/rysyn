use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstrumentType {
    None,
    SimpleSine,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub bpm: f32,
    pub is_playing: bool,
    pub playhead_pos: f64, // seconds
    pub tracks: Vec<Track>,
    pub patterns: Vec<Pattern>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "New Project".to_string(),
            bpm: 120.0,
            is_playing: false,
            playhead_pos: 0.0,
            tracks: Vec::new(),
            patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: u32,
    pub name: String,
    pub volume: f32, // Linear gain: 0.0 to 1.0+
    pub pan: f32,    // -1.0 (Left) to 1.0 (Right)
    pub mute: bool,
    pub solo: bool,
    pub items: Vec<AudioItem>,
    pub pattern_instances: Vec<PatternInstance>,
    pub instrument: InstrumentType,
}

impl Track {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            volume: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            items: Vec::new(),
            pattern_instances: Vec::new(),
            instrument: InstrumentType::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioItem {
    pub id: u32,
    pub start_time: f64, // Seconds
    pub duration: f64,   // Seconds
    pub source_path: String,
    pub start_offset: f64, // Start point in source file (seconds)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: u32,
    pub name: String,
    pub length: f64, // beats (e.g., 4.0 for 1 bar)
    pub notes: Vec<NoteEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEvent {
    pub start_time: f64, // beats (relative to pattern start)
    pub duration: f64,   // beats
    pub key: u8,         // MIDI note number (0-127)
    pub velocity: u8,    // 0-127
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInstance {
    pub id: u32,
    pub pattern_id: u32,
    pub start_time: f64, // seconds (on main timeline)
}

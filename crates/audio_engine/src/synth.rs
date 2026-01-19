use std::collections::HashMap;
use rysyn_project::{Pattern, PatternInstance};

pub struct GlobalSynthState {
    phases: HashMap<u32, f32>, // TrackId -> Phase
}

impl GlobalSynthState {
    pub fn new() -> Self {
        Self { phases: HashMap::new() }
    }

    pub fn get_phase(&self, track_id: u32) -> f32 {
        *self.phases.get(&track_id).unwrap_or(&0.0)
    }

    pub fn set_phase(&mut self, track_id: u32, phase: f32) {
        self.phases.insert(track_id, phase);
    }
}

pub fn note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

pub fn render_track_synth(
    _track_id: u32,
    bpm: f32,
    current_time: f64, // seconds
    instances: &[PatternInstance],
    patterns: &[Pattern],
) -> Option<f32> {
    // RETURNS FREQUENCY if a note is playing, None otherwise.
    
    // 1. Convert Global Time to Beats
    // For simplicity, let's assume Time 0 = Beat 0.
    // Complex tempo maps are Phase 10 shit.
    let beats_per_second = bpm / 60.0;
    let current_beat = current_time * beats_per_second as f64;

    // 2. Find Active Pattern
    for instance in instances {
        let instance_start_beat = instance.start_time * beats_per_second as f64;
        
        // Check if we are physically past the start of this instance
        if current_beat >= instance_start_beat {
             // Find Pattern
             if let Some(pattern) = patterns.iter().find(|p| p.id == instance.pattern_id) {
                 // Local beat time
                 let local_beat = current_beat - instance_start_beat;
                 
                 // Loop logic? For now, no looping patterns. 1-shot.
                 if local_beat < pattern.length {
                     // Inside Pattern. Find Note.
                     for note in &pattern.notes {
                         if local_beat >= note.start_time && local_beat < (note.start_time + note.duration) {
                             return Some(note_to_freq(note.key));
                         }
                     }
                 }
             }
        }
    }
    
    None
}

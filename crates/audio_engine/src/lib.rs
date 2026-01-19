pub mod io;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rysyn_project::Project;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    stream: Option<cpal::Stream>,
    project: Arc<Mutex<Project>>,
    // Cache for loaded audio samples: Path -> Vec<f32> (Interleaved or mono)
    sample_cache: Arc<Mutex<HashMap<String, Vec<f32>>>>, 
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            stream: None,
            project: Arc::new(Mutex::new(Project::default())),
            sample_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_project(&self) -> Arc<Mutex<Project>> {
        self.project.clone()
    }

    // New method to load file for use
    pub fn load_audio_file(&self, path: &str) -> Result<(), String> {
        let mut cache = self.sample_cache.lock().unwrap();
        if cache.contains_key(path) {
            return Ok(());
        }

        let samples = io::loader::AudioLoader::load_file(path)?;
        println!("Loaded {} samples from {}", samples.len(), path);
        cache.insert(path.to_string(), samples);
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("No output device available")?;
        let config = device.default_output_config().map_err(|e| e.to_string())?;

        println!("Using audio device: {}", device.description().map(|d| d.to_string()).unwrap_or_else(|_| "Unknown Device".to_string()));

        let project_ref = self.project.clone();
        let cache_ref = self.sample_cache.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), project_ref, cache_ref)?,
            _ => return Err("Only F32 sample format is currently supported for MVP".to_string()),
        };

        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None; 
    }
}

fn run<T>(
    device: &cpal::Device, 
    config: &cpal::StreamConfig,
    project: Arc<Mutex<Project>>,
    cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
) -> Result<cpal::Stream, String>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut project_guard = project.lock().unwrap(); // WARNING: Blocking
            let cache_guard = cache.lock().unwrap();         // WARNING: Blocking
            
            // Advance transport if playing
            let playing = project_guard.is_playing;
            let mut current_pos = project_guard.playhead_pos;

            // Collect active items based on playhead position
            // For each active item, we need to know:
            // 1. Where in the item buffer are we? (playhead - item.start_time + item.start_offset)
            // 2. Track volume/pan
            
            let dt = 1.0 / sample_rate;

            for frame in data.chunks_mut(channels) {
                let mut mixed_sample_l = 0.0;
                let mut mixed_sample_r = 0.0;

                if playing {
                     for track in &project_guard.tracks {
                        if track.mute { continue; }
                        
                        let vol = track.volume;
                        let pan = track.pan;

                        // Simple pan law (-1.0 to 1.0)
                        let l_gain = vol * (if pan > 0.0 { 1.0 - pan } else { 1.0 });
                        let r_gain = vol * (if pan < 0.0 { 1.0 + pan } else { 1.0 });

                        for item in &track.items {
                            // Check overlap
                            if current_pos >= item.start_time && current_pos < (item.start_time + item.duration) {
                                if let Some(buffer) = cache_guard.get(&item.source_path) {
                                    // Calculate sample index
                                    let item_time = current_pos - item.start_time + item.start_offset;
                                    let sample_idx = (item_time * 44100.0) as usize; // Assuming 44.1k source for MVP

                                    // Very crude mono read
                                    if sample_idx < buffer.len() {
                                        let s = buffer[sample_idx];
                                        mixed_sample_l += s * l_gain;
                                        mixed_sample_r += s * r_gain;
                                    }
                                }
                            }
                        }
                     }
                    
                     current_pos += dt as f64;
                } else {
                    // Not playing -> Silence
                }

                // Write output
                if channels >= 2 {
                    frame[0] = cpal::Sample::from_sample(mixed_sample_l);
                    frame[1] = cpal::Sample::from_sample(mixed_sample_r);
                } else {
                    // Mono downmix
                    frame[0] = cpal::Sample::from_sample((mixed_sample_l + mixed_sample_r) * 0.5);
                }
            }

            // Sync back playhead to project state (approximate)
            if playing {
                project_guard.playhead_pos = current_pos;
            }
        },
        err_fn,
        None
    ).map_err(|e| e.to_string())?;

    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_constructs() {
        let _engine = AudioEngine::new();
        assert!(true);
    }
}

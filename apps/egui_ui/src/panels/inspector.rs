//! Inspector Panel
//!
//! Shows properties of selected items (tracks, clips, plugins)

use eframe::egui::{self, Color32, RichText};
use rysyn_ffi_bridge::{Command, StateSnapshot, TrackState, ClipState};

#[derive(Default)]
pub struct InspectorPanel {
    // For editing properties
    edit_name: String,
}

impl InspectorPanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        mut send_cmd: impl FnMut(Command),
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Find selected items
                let selected_track = state.tracks.iter().find(|t| t.is_selected);
                let selected_clip = state.clips.iter().find(|c| c.is_selected);
                
                match (selected_track, selected_clip) {
                    (_, Some(clip)) => {
                        self.show_clip_inspector(ui, clip, state, &mut send_cmd);
                    }
                    (Some(track), None) => {
                        self.show_track_inspector(ui, track, state, &mut send_cmd);
                    }
                    (None, None) => {
                        self.show_project_inspector(ui, state, &mut send_cmd);
                    }
                }
            });
    }
    
    fn show_track_inspector(
        &mut self,
        ui: &mut egui::Ui,
        track: &TrackState,
        _state: &StateSnapshot,
        send_cmd: &mut impl FnMut(Command),
    ) {
        ui.heading("Track");
        ui.separator();
        
        // Name
        egui::Grid::new("track_inspector_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Name:");
                let mut name = track.name.clone();
                if ui.text_edit_singleline(&mut name).changed() {
                    send_cmd(Command::RenameTrack { 
                        track_id: track.id,
                        name
                    });
                }
                ui.end_row();
                
                ui.label("Type:");
                ui.label(if track.is_midi { "MIDI" } else { "Audio" });
                ui.end_row();
                
                ui.label("Volume:");
                let mut vol = track.volume;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=1.5).suffix(" dB")).changed() {
                    send_cmd(Command::SetTrackVolume {
                        track_id: track.id,
                        volume: vol
                    });
                }
                ui.end_row();
                
                ui.label("Pan:");
                let mut pan = track.pan;
                if ui.add(egui::Slider::new(&mut pan, -1.0..=1.0)).changed() {
                    send_cmd(Command::SetTrackPan {
                        track_id: track.id,
                        pan
                    });
                }
                ui.end_row();
                
                ui.label("Mute:");
                let mut muted = track.is_muted;
                if ui.checkbox(&mut muted, "").changed() {
                    send_cmd(Command::SetTrackMute {
                        track_id: track.id,
                        muted
                    });
                }
                ui.end_row();
                
                ui.label("Solo:");
                let mut soloed = track.is_soloed;
                if ui.checkbox(&mut soloed, "").changed() {
                    send_cmd(Command::SetTrackSolo {
                        track_id: track.id,
                        soloed
                    });
                }
                ui.end_row();
                
                if track.is_midi {
                    ui.label("Record Arm:");
                    let mut armed = track.is_armed;
                    if ui.checkbox(&mut armed, "").changed() {
                        send_cmd(Command::SetTrackArm {
                            track_id: track.id,
                            armed
                        });
                    }
                    ui.end_row();
                }
            });
        
        ui.add_space(16.0);
        
        // Color picker
        ui.horizontal(|ui| {
            ui.label("Color:");
            
            // Extract RGBA from u32
            let r = ((track.color >> 24) & 0xFF) as u8;
            let g = ((track.color >> 16) & 0xFF) as u8;
            let b = ((track.color >> 8) & 0xFF) as u8;
            let mut color = [r, g, b];
            
            if ui.color_edit_button_srgb(&mut color).changed() {
                let new_color = ((color[0] as u32) << 24) 
                              | ((color[1] as u32) << 16) 
                              | ((color[2] as u32) << 8) 
                              | 0xFF;
                send_cmd(Command::SetTrackColor {
                    track_id: track.id,
                    color: new_color
                });
            }
        });
        
        ui.add_space(16.0);
        
        // Plugin chain
        ui.collapsing("Plugins", |ui| {
            for (slot, plugin) in track.plugins.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", slot + 1));
                    
                    if let Some(name) = plugin {
                        ui.label(name);
                        if ui.small_button("✕").clicked() {
                            send_cmd(Command::RemovePlugin {
                                track_id: track.id,
                                slot: slot as u32
                            });
                        }
                        if ui.small_button("⚙").clicked() {
                            send_cmd(Command::OpenPluginEditor {
                                track_id: track.id,
                                slot: slot as u32
                            });
                        }
                    } else {
                        ui.label(RichText::new("Empty").color(Color32::GRAY));
                    }
                });
            }
            
            if ui.button("+ Add Plugin").clicked() {
                // Would open plugin browser
            }
        });
        
        ui.add_space(16.0);
        
        // Input/Output routing
        ui.collapsing("I/O Routing", |ui| {
            ui.label(RichText::new("Input:").strong());
            ui.label(&track.input_name);
            
            ui.label(RichText::new("Output:").strong());
            ui.label(&track.output_name);
        });
    }
    
    fn show_clip_inspector(
        &mut self,
        ui: &mut egui::Ui,
        clip: &ClipState,
        state: &StateSnapshot,
        send_cmd: &mut impl FnMut(Command),
    ) {
        ui.heading(if clip.is_midi { "MIDI Clip" } else { "Audio Clip" });
        ui.separator();
        
        egui::Grid::new("clip_inspector_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Name:");
                let mut name = clip.name.clone();
                if ui.text_edit_singleline(&mut name).changed() {
                    send_cmd(Command::RenameClip { 
                        clip_id: clip.id,
                        name
                    });
                }
                ui.end_row();
                
                ui.label("Track:");
                let track_name = state.tracks.iter()
                    .find(|t| t.id == clip.track_id)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown");
                ui.label(track_name);
                ui.end_row();
                
                ui.label("Start:");
                let mut start = clip.start_beats;
                if ui.add(
                    egui::DragValue::new(&mut start)
                        .speed(0.1)
                        .suffix(" beats")
                ).changed() {
                    send_cmd(Command::MoveClip {
                        clip_id: clip.id,
                        start_beats: start,
                        track_id: None
                    });
                }
                ui.end_row();
                
                ui.label("Length:");
                let mut length = clip.length_beats;
                if ui.add(
                    egui::DragValue::new(&mut length)
                        .speed(0.1)
                        .suffix(" beats")
                        .range(0.1..=1000.0)
                ).changed() {
                    send_cmd(Command::ResizeClip {
                        clip_id: clip.id,
                        length_beats: length
                    });
                }
                ui.end_row();
                
                ui.label("Offset:");
                ui.label(format!("{:.2} beats", clip.offset_beats));
                ui.end_row();
            });
        
        ui.add_space(16.0);
        
        // Color picker
        ui.horizontal(|ui| {
            ui.label("Color:");
            
            let r = ((clip.color >> 24) & 0xFF) as u8;
            let g = ((clip.color >> 16) & 0xFF) as u8;
            let b = ((clip.color >> 8) & 0xFF) as u8;
            let mut color = [r, g, b];
            
            if ui.color_edit_button_srgb(&mut color).changed() {
                let new_color = ((color[0] as u32) << 24) 
                              | ((color[1] as u32) << 16) 
                              | ((color[2] as u32) << 8) 
                              | 0xFF;
                send_cmd(Command::SetClipColor {
                    clip_id: clip.id,
                    color: new_color
                });
            }
        });
        
        ui.add_space(16.0);
        
        // Clip-specific options
        if clip.is_midi {
            ui.collapsing("MIDI Options", |ui| {
                ui.label("Transpose:");
                let mut transpose = 0i32;
                ui.add(egui::DragValue::new(&mut transpose).speed(1).suffix(" st"));
                
                ui.label("Velocity:");
                let mut velocity_scale = 100i32;
                ui.add(egui::DragValue::new(&mut velocity_scale).speed(1).suffix("%"));
            });
        } else {
            ui.collapsing("Audio Options", |ui| {
                ui.label("Gain:");
                let mut gain_db = 0.0f32;
                ui.add(egui::Slider::new(&mut gain_db, -24.0..=24.0).suffix(" dB"));
                
                ui.label("Fade In:");
                let mut fade_in = 0.0f32;
                ui.add(egui::DragValue::new(&mut fade_in).speed(0.01).suffix(" beats"));
                
                ui.label("Fade Out:");
                let mut fade_out = 0.0f32;
                ui.add(egui::DragValue::new(&mut fade_out).speed(0.01).suffix(" beats"));
                
                ui.checkbox(&mut false, "Warp / Time Stretch");
            });
        }
        
        ui.add_space(16.0);
        
        // Actions
        ui.horizontal(|ui| {
            if ui.button("Duplicate").clicked() {
                send_cmd(Command::DuplicateClip { clip_id: clip.id });
            }
            if ui.button("Delete").clicked() {
                send_cmd(Command::DeleteClip { clip_id: clip.id });
            }
        });
    }
    
    fn show_project_inspector(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        send_cmd: &mut impl FnMut(Command),
    ) {
        ui.heading("Project");
        ui.separator();
        
        egui::Grid::new("project_inspector_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Name:");
                let mut name = state.project_name.clone();
                if ui.text_edit_singleline(&mut name).changed() {
                    send_cmd(Command::SetProjectName { name });
                }
                ui.end_row();
                
                ui.label("BPM:");
                let mut bpm = state.transport.tempo;
                if ui.add(
                    egui::DragValue::new(&mut bpm)
                        .speed(0.1)
                        .range(20.0..=300.0)
                ).changed() {
                    send_cmd(Command::SetTempo { bpm });
                }
                ui.end_row();
                
                ui.label("Time Signature:");
                let mut num = state.transport.time_sig_num;
                let mut denom = state.transport.time_sig_denom;
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::DragValue::new(&mut num)
                            .range(1..=16)
                    ).changed() || ui.add(
                        egui::DragValue::new(&mut denom)
                            .range(1..=16)
                    ).changed() {
                        send_cmd(Command::SetTimeSignature {
                            numerator: num,
                            denominator: denom
                        });
                    }
                });
                ui.end_row();
                
                ui.label("Sample Rate:");
                ui.label(format!("{} Hz", state.sample_rate));
                ui.end_row();
                
                ui.label("Tracks:");
                ui.label(format!("{}", state.tracks.len()));
                ui.end_row();
                
                ui.label("Clips:");
                ui.label(format!("{}", state.clips.len()));
                ui.end_row();
            });
        
        ui.add_space(16.0);
        
        ui.collapsing("Audio Settings", |ui| {
            ui.label(RichText::new("Output Device:").strong());
            ui.label(&state.audio_device_name);
            
            ui.label(RichText::new("Buffer Size:").strong());
            ui.label(format!("{} samples", state.buffer_size));
            
            ui.label(RichText::new("Latency:").strong());
            let latency_ms = (state.buffer_size as f64 / state.sample_rate as f64) * 1000.0;
            ui.label(format!("{:.1} ms", latency_ms));
            
            if ui.button("⚙ Audio Settings...").clicked() {
                send_cmd(Command::OpenAudioSettings);
            }
        });
        
        ui.add_space(8.0);
        
        ui.label(
            RichText::new("Select a track or clip to see its properties")
                .size(11.0)
                .color(Color32::GRAY)
        );
    }
}

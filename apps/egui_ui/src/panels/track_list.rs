//! Track List Panel
//!
//! Track headers with name, volume, pan, mute, solo controls

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot, TrackState};

const TRACK_HEIGHT: f32 = 80.0;

#[derive(Default)]
pub struct TrackListPanel {
    renaming_track: Option<u32>,
    rename_buffer: String,
}

impl TrackListPanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        mut send_cmd: impl FnMut(Command),
    ) {
        // Add track button at top
        ui.horizontal(|ui| {
            if ui.button("+ Audio").clicked() {
                send_cmd(Command::CreateTrack { 
                    name: "Audio".to_string(),
                    is_midi: false 
                });
            }
            if ui.button("+ MIDI").clicked() {
                send_cmd(Command::CreateTrack { 
                    name: "MIDI".to_string(),
                    is_midi: true 
                });
            }
        });
        
        ui.add_space(4.0);
        ui.separator();
        
        // Track list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for track in &state.tracks {
                    self.draw_track_header(ui, track, &mut send_cmd);
                }
            });
    }
    
    fn draw_track_header(
        &mut self,
        ui: &mut egui::Ui,
        track: &TrackState,
        send_cmd: &mut impl FnMut(Command),
    ) {
        let available_width = ui.available_width();
        
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, TRACK_HEIGHT),
            Sense::click()
        );
        
        // Background
        let bg_color = if track.is_selected {
            Color32::from_rgb(50, 60, 80)
        } else {
            Color32::from_rgb(40, 40, 45)
        };
        
        ui.painter().rect_filled(rect, 4.0, bg_color);
        
        // Border
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(60, 60, 65))
        );
        
        // Track color indicator
        let color = Color32::from_rgba_unmultiplied(
            ((track.color >> 24) & 0xFF) as u8,
            ((track.color >> 16) & 0xFF) as u8,
            ((track.color >> 8) & 0xFF) as u8,
            255
        );
        
        let color_bar = Rect::from_min_size(
            rect.min,
            Vec2::new(4.0, TRACK_HEIGHT)
        );
        ui.painter().rect_filled(color_bar, 4.0, color);
        
        // Content area
        let content_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 8.0, rect.min.y + 4.0),
            Pos2::new(rect.max.x - 4.0, rect.max.y - 4.0)
        );
        
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical(|ui| {
                // Track name (editable)
                ui.horizontal(|ui| {
                    if self.renaming_track == Some(track.id) {
                        let edit = ui.text_edit_singleline(&mut self.rename_buffer);
                        if edit.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            send_cmd(Command::RenameTrack { 
                                track_id: track.id,
                                name: self.rename_buffer.clone()
                            });
                            self.renaming_track = None;
                        }
                    } else {
                        let name_response = ui.add(
                            egui::Label::new(
                                RichText::new(&track.name)
                                    .color(Color32::WHITE)
                                    .strong()
                            ).sense(Sense::click())
                        );
                        
                        if name_response.double_clicked() {
                            self.renaming_track = Some(track.id);
                            self.rename_buffer = track.name.clone();
                        }
                    }
                    
                    // Track type indicator
                    let type_text = if track.is_midi { "MIDI" } else { "AUD" };
                    ui.label(
                        RichText::new(type_text)
                            .size(9.0)
                            .color(Color32::from_rgb(150, 150, 150))
                    );
                });
                
                ui.add_space(4.0);
                
                // Mute / Solo buttons
                ui.horizontal(|ui| {
                    let mute_color = if track.is_muted {
                        Color32::from_rgb(255, 80, 80)
                    } else {
                        Color32::from_rgb(100, 100, 100)
                    };
                    
                    if ui.add(
                        egui::Button::new(RichText::new("M").color(mute_color))
                            .min_size(Vec2::new(24.0, 20.0))
                    ).clicked() {
                        send_cmd(Command::SetTrackMute { 
                            track_id: track.id,
                            muted: !track.is_muted
                        });
                    }
                    
                    let solo_color = if track.is_soloed {
                        Color32::from_rgb(255, 200, 50)
                    } else {
                        Color32::from_rgb(100, 100, 100)
                    };
                    
                    if ui.add(
                        egui::Button::new(RichText::new("S").color(solo_color))
                            .min_size(Vec2::new(24.0, 20.0))
                    ).clicked() {
                        send_cmd(Command::SetTrackSolo { 
                            track_id: track.id,
                            soloed: !track.is_soloed
                        });
                    }
                    
                    // Record arm (MIDI tracks only)
                    if track.is_midi {
                        let rec_color = if track.is_armed {
                            Color32::from_rgb(255, 50, 50)
                        } else {
                            Color32::from_rgb(100, 100, 100)
                        };
                        
                        if ui.add(
                            egui::Button::new(RichText::new("R").color(rec_color))
                                .min_size(Vec2::new(24.0, 20.0))
                        ).clicked() {
                            send_cmd(Command::SetTrackArm { 
                                track_id: track.id,
                                armed: !track.is_armed
                            });
                        }
                    }
                });
                
                // Volume slider
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Vol")
                            .size(10.0)
                            .color(Color32::from_rgb(150, 150, 150))
                    );
                    
                    let mut vol = track.volume;
                    let vol_slider = ui.add(
                        egui::Slider::new(&mut vol, 0.0..=1.5)
                            .show_value(false)
                            .custom_formatter(|v, _| format!("{:.1} dB", 20.0 * (v as f64).log10()))
                    );
                    
                    if vol_slider.changed() {
                        send_cmd(Command::SetTrackVolume {
                            track_id: track.id,
                            volume: vol
                        });
                    }
                    
                    // dB display
                    let db = if track.volume > 0.0 {
                        20.0 * (track.volume as f64).log10()
                    } else {
                        -60.0
                    };
                    ui.label(
                        RichText::new(format!("{:.1}", db))
                            .size(10.0)
                            .color(Color32::from_rgb(180, 180, 180))
                    );
                });
                
                // Pan knob (simplified as slider)
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Pan")
                            .size(10.0)
                            .color(Color32::from_rgb(150, 150, 150))
                    );
                    
                    let mut pan = track.pan;
                    let pan_slider = ui.add(
                        egui::Slider::new(&mut pan, -1.0..=1.0)
                            .show_value(false)
                    );
                    
                    if pan_slider.changed() {
                        send_cmd(Command::SetTrackPan {
                            track_id: track.id,
                            pan
                        });
                    }
                    
                    // Pan display
                    let pan_text = if track.pan.abs() < 0.01 {
                        "C".to_string()
                    } else if track.pan < 0.0 {
                        format!("L{:.0}", track.pan.abs() * 100.0)
                    } else {
                        format!("R{:.0}", track.pan * 100.0)
                    };
                    ui.label(
                        RichText::new(pan_text)
                            .size(10.0)
                            .color(Color32::from_rgb(180, 180, 180))
                    );
                });
            });
        });
        
        // Context menu
        response.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                self.renaming_track = Some(track.id);
                self.rename_buffer = track.name.clone();
                ui.close_menu();
            }
            if ui.button("Duplicate").clicked() {
                send_cmd(Command::DuplicateTrack { track_id: track.id });
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Delete").clicked() {
                send_cmd(Command::DeleteTrack { track_id: track.id });
                ui.close_menu();
            }
        });
        
        // Click to select
        if response.clicked() {
            send_cmd(Command::SelectTrack { 
                track_id: track.id,
                exclusive: !ui.input(|i| i.modifiers.ctrl)
            });
        }
    }
}

//! Mixer Panel
//!
//! Channel strips with faders, meters, sends

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot, TrackState, MeterLevels};

const CHANNEL_WIDTH: f32 = 80.0;
const METER_WIDTH: f32 = 12.0;
const FADER_WIDTH: f32 = 20.0;

#[derive(Default)]
pub struct MixerPanel;

impl MixerPanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        mut send_cmd: impl FnMut(Command),
    ) {
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Track channels
                    for track in &state.tracks {
                        self.draw_channel_strip(ui, track, &state.meters, &mut send_cmd);
                        ui.add_space(4.0);
                    }
                    
                    // Master channel
                    self.draw_master_channel(ui, state, &mut send_cmd);
                });
            });
    }
    
    fn draw_channel_strip(
        &mut self,
        ui: &mut egui::Ui,
        track: &TrackState,
        meters: &MeterLevels,
        send_cmd: &mut impl FnMut(Command),
    ) {
        let available_height = ui.available_height();
        
        egui::Frame::new()
            .fill(Color32::from_rgb(40, 40, 45))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 65)))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.allocate_ui(Vec2::new(CHANNEL_WIDTH, available_height), |ui| {
                    ui.vertical(|ui| {
                        // Track name
                        ui.horizontal(|ui| {
                            let color = Color32::from_rgba_unmultiplied(
                                ((track.color >> 24) & 0xFF) as u8,
                                ((track.color >> 16) & 0xFF) as u8,
                                ((track.color >> 8) & 0xFF) as u8,
                                255
                            );
                            
                            // Color dot
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::splat(8.0),
                                Sense::hover()
                            );
                            ui.painter().circle_filled(rect.center(), 4.0, color);
                            
                            ui.label(
                                RichText::new(&track.name)
                                    .size(11.0)
                                    .color(Color32::WHITE)
                            );
                        });
                        
                        ui.add_space(4.0);
                        
                        // Mute/Solo/Arm buttons
                        ui.horizontal(|ui| {
                            let mute_color = if track.is_muted {
                                Color32::from_rgb(255, 80, 80)
                            } else {
                                Color32::from_rgb(80, 80, 80)
                            };
                            
                            if ui.add(
                                egui::Button::new(RichText::new("M").size(10.0).color(mute_color))
                                    .min_size(Vec2::new(20.0, 18.0))
                            ).clicked() {
                                send_cmd(Command::SetTrackMute { 
                                    track_id: track.id,
                                    muted: !track.is_muted
                                });
                            }
                            
                            let solo_color = if track.is_soloed {
                                Color32::from_rgb(255, 200, 50)
                            } else {
                                Color32::from_rgb(80, 80, 80)
                            };
                            
                            if ui.add(
                                egui::Button::new(RichText::new("S").size(10.0).color(solo_color))
                                    .min_size(Vec2::new(20.0, 18.0))
                            ).clicked() {
                                send_cmd(Command::SetTrackSolo { 
                                    track_id: track.id,
                                    soloed: !track.is_soloed
                                });
                            }
                            
                            if track.is_midi {
                                let rec_color = if track.is_armed {
                                    Color32::from_rgb(255, 50, 50)
                                } else {
                                    Color32::from_rgb(80, 80, 80)
                                };
                                
                                if ui.add(
                                    egui::Button::new(RichText::new("R").size(10.0).color(rec_color))
                                        .min_size(Vec2::new(20.0, 18.0))
                                ).clicked() {
                                    send_cmd(Command::SetTrackArm { 
                                        track_id: track.id,
                                        armed: !track.is_armed
                                    });
                                }
                            }
                        });
                        
                        ui.add_space(8.0);
                        
                        // Pan knob
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Pan").size(9.0).color(Color32::GRAY));
                        });
                        
                        let mut pan = track.pan;
                        if ui.add(
                            egui::Slider::new(&mut pan, -1.0..=1.0)
                                .show_value(false)
                        ).changed() {
                            send_cmd(Command::SetTrackPan {
                                track_id: track.id,
                                pan
                            });
                        }
                        
                        let pan_text = if track.pan.abs() < 0.01 {
                            "C".to_string()
                        } else if track.pan < 0.0 {
                            format!("L{:.0}", track.pan.abs() * 100.0)
                        } else {
                            format!("R{:.0}", track.pan * 100.0)
                        };
                        ui.label(
                            RichText::new(pan_text)
                                .size(9.0)
                                .color(Color32::from_rgb(180, 180, 180))
                        );
                        
                        ui.add_space(8.0);
                        
                        // Fader + Meter area
                        let remaining = ui.available_height() - 40.0;
                        
                        ui.horizontal(|ui| {
                            // Meter
                            self.draw_meter(ui, remaining, meters.track_levels_l.get(track.id as usize).copied().unwrap_or(0.0),
                                          meters.track_levels_r.get(track.id as usize).copied().unwrap_or(0.0));
                            
                            // Fader
                            let mut vol = track.volume;
                            ui.vertical(|ui| {
                                let fader_response = ui.add(
                                    egui::Slider::new(&mut vol, 0.0..=1.5)
                                        .vertical()
                                        .show_value(false)
                                );
                                
                                if fader_response.changed() {
                                    send_cmd(Command::SetTrackVolume {
                                        track_id: track.id,
                                        volume: vol
                                    });
                                }
                            });
                        });
                        
                        // dB display
                        let db = if track.volume > 0.0 {
                            20.0 * (track.volume as f64).log10()
                        } else {
                            -60.0
                        };
                        
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new(format!("{:.1} dB", db))
                                    .size(10.0)
                                    .color(Color32::WHITE)
                            );
                        });
                    });
                });
            });
    }
    
    fn draw_master_channel(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        send_cmd: &mut impl FnMut(Command),
    ) {
        let available_height = ui.available_height();
        
        egui::Frame::new()
            .fill(Color32::from_rgb(50, 45, 45))
            .stroke(Stroke::new(1.0, Color32::from_rgb(80, 70, 70)))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.allocate_ui(Vec2::new(CHANNEL_WIDTH + 20.0, available_height), |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("MASTER")
                                    .size(11.0)
                                    .color(Color32::from_rgb(255, 200, 200))
                                    .strong()
                            );
                        });
                        
                        ui.add_space(28.0); // Space where M/S buttons would be
                        
                        // Pan (disabled for master)
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Pan").size(9.0).color(Color32::GRAY));
                        });
                        ui.add_enabled(false, egui::Slider::new(&mut 0.0f32, -1.0..=1.0).show_value(false));
                        ui.label(RichText::new("C").size(9.0).color(Color32::GRAY));
                        
                        ui.add_space(8.0);
                        
                        // Fader + Meter
                        let remaining = ui.available_height() - 40.0;
                        
                        ui.horizontal(|ui| {
                            // Master meter (stereo)
                            self.draw_meter(ui, remaining, state.meters.master_l, state.meters.master_r);
                            
                            // Master fader
                            let mut vol = state.master_volume;
                            ui.vertical(|ui| {
                                let fader_response = ui.add(
                                    egui::Slider::new(&mut vol, 0.0..=1.5)
                                        .vertical()
                                        .show_value(false)
                                );
                                
                                if fader_response.changed() {
                                    send_cmd(Command::SetMasterVolume { volume: vol });
                                }
                            });
                        });
                        
                        // dB display
                        let db = if state.master_volume > 0.0 {
                            20.0 * (state.master_volume as f64).log10()
                        } else {
                            -60.0
                        };
                        
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new(format!("{:.1} dB", db))
                                    .size(10.0)
                                    .color(Color32::WHITE)
                            );
                        });
                    });
                });
            });
    }
    
    fn draw_meter(&self, ui: &mut egui::Ui, height: f32, level_l: f32, level_r: f32) {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(METER_WIDTH * 2.0 + 2.0, height),
            Sense::hover()
        );
        
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 20, 22));
        
        // Left channel
        let left_rect = Rect::from_min_size(
            rect.min,
            Vec2::new(METER_WIDTH, height)
        );
        self.draw_meter_channel(painter, left_rect, level_l);
        
        // Right channel
        let right_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + METER_WIDTH + 2.0, rect.min.y),
            Vec2::new(METER_WIDTH, height)
        );
        self.draw_meter_channel(painter, right_rect, level_r);
    }
    
    fn draw_meter_channel(&self, painter: &egui::Painter, rect: Rect, level: f32) {
        // Convert to dB and normalize
        let db = if level > 0.0 {
            20.0 * (level as f64).log10()
        } else {
            -60.0
        };
        
        // Map -60dB to +6dB to 0.0 to 1.0
        let normalized = ((db + 60.0) / 66.0).clamp(0.0, 1.0) as f32;
        
        // Calculate fill height
        let fill_height = rect.height() * normalized;
        
        // Gradient fill
        let fill_rect = Rect::from_min_max(
            Pos2::new(rect.min.x, rect.max.y - fill_height),
            rect.max
        );
        
        // Color based on level
        let color = if db > 0.0 {
            Color32::from_rgb(255, 50, 50) // Red for clipping
        } else if db > -6.0 {
            Color32::from_rgb(255, 200, 50) // Yellow for hot
        } else {
            Color32::from_rgb(50, 200, 100) // Green for normal
        };
        
        painter.rect_filled(fill_rect, 0.0, color);
        
        // Peak indicator
        // (In a real implementation, this would track peak hold)
    }
}

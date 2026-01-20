//! Mixer Panel
//!
//! Channel strips with faders, meters, sends

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot, TrackState, MeterLevels};
use crate::theme::DawColors;

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
                    ui.spacing_mut().item_spacing.x = 4.0;
                    
                    // Track channels
                    for track in &state.tracks {
                        self.draw_channel_strip(ui, track, &state.meters, &mut send_cmd);
                    }
                    
                    // Spacer
                    ui.add_space(10.0);
                    
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
        egui::Frame::new()
            .fill(ui.visuals().window_fill)
            .stroke(ui.visuals().window_stroke)
            .inner_margin(4.0)
            .corner_radius(4.0)
            .show(ui, |ui| {
                let available_height = ui.available_height();
                ui.allocate_ui(Vec2::new(CHANNEL_WIDTH, available_height), |ui| {
                    ui.vertical(|ui| {
                        // Color label at top
                        let track_color = Color32::from_rgba_unmultiplied(
                                ((track.color >> 24) & 0xFF) as u8,
                                ((track.color >> 16) & 0xFF) as u8,
                                ((track.color >> 8) & 0xFF) as u8,
                                255
                            );
                        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(CHANNEL_WIDTH, 4.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, track_color);
                        
                        ui.add_space(4.0);
                        
                        // Track name
                        ui.label(
                            RichText::new(&track.name)
                                .size(12.0)
                                .color(ui.visuals().text_color())
                                .strong()
                        );
                        
                        ui.separator();
                        
                        // Mute/Solo/Arm buttons
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            // Mute
                            let mute_color = if track.is_muted {
                                DawColors::METER_RED
                            } else {
                                ui.visuals().weak_text_color()
                            };
                            if ui.add(egui::Button::new(RichText::new("M").size(12.0).strong().color(mute_color))
                                .min_size(Vec2::new(24.0, 24.0))
                                .frame(true)
                            ).clicked() {
                                send_cmd(Command::SetTrackMute { track_id: track.id, muted: !track.is_muted });
                            }
                            
                            // Solo
                            let solo_color = if track.is_soloed {
                                DawColors::METER_YELLOW
                            } else {
                                ui.visuals().weak_text_color()
                            };
                            if ui.add(egui::Button::new(RichText::new("S").size(12.0).strong().color(solo_color))
                                .min_size(Vec2::new(24.0, 24.0))
                                .frame(true)
                            ).clicked() {
                                send_cmd(Command::SetTrackSolo { track_id: track.id, soloed: !track.is_soloed });
                            }
                            
                            // Record Arm
                            if track.is_midi {
                                let rec_color = if track.is_armed {
                                    DawColors::RECORD
                                } else {
                                    ui.visuals().weak_text_color()
                                };
                                if ui.add(egui::Button::new(RichText::new("R").size(12.0).strong().color(rec_color))
                                    .min_size(Vec2::new(24.0, 24.0))
                                    .frame(true)
                                ).clicked() {
                                    send_cmd(Command::SetTrackArm { track_id: track.id, armed: !track.is_armed });
                                }
                            }
                        });
                        
                        ui.add_space(8.0);
                        
                        // Pan knob
                        ui.label(RichText::new("PAN").size(9.0).color(ui.visuals().weak_text_color()));
                        
                        let mut pan = track.pan;
                        if ui.add(
                            egui::Slider::new(&mut pan, -1.0..=1.0)
                                .show_value(false)
                                .handle_shape(egui::style::HandleShape::Circle)
                        ).changed() {
                            send_cmd(Command::SetTrackPan { track_id: track.id, pan });
                        }
                        
                        let pan_text = if track.pan.abs() < 0.01 { "C".to_string() }
                                       else if track.pan < 0.0 { format!("L{:.0}", track.pan.abs() * 100.0) }
                                       else { format!("R{:.0}", track.pan * 100.0) };
                        ui.label(RichText::new(pan_text).size(9.0).color(ui.visuals().weak_text_color()));
                        
                        ui.add_space(8.0);
                        
                        // Fader + Meter area
                        let remaining = (ui.available_height() - 50.0).max(10.0);
                        ui.horizontal(|ui| {
                            // Meter
                            self.draw_meter(ui, remaining, 
                                meters.track_levels_l.get(track.id as usize).copied().unwrap_or(0.0),
                                meters.track_levels_r.get(track.id as usize).copied().unwrap_or(0.0));
                            
                            // Fader
                            let mut vol = track.volume;
                            ui.vertical(|ui| {
                                let fader_response = ui.add(
                                    egui::Slider::new(&mut vol, 0.0..=1.5)
                                        .vertical()
                                        .show_value(false)
                                        .step_by(0.01)
                                );
                                
                                if fader_response.changed() {
                                    send_cmd(Command::SetTrackVolume { track_id: track.id, volume: vol });
                                }
                                
                                // Double click to reset
                                if fader_response.double_clicked() {
                                    send_cmd(Command::SetTrackVolume { track_id: track.id, volume: 1.0 });
                                }
                            });
                        });
                        
                        // dB display with input field
                        // For now just label
                        let db = if track.volume > 0.0 { 20.0 * (track.volume as f64).log10() } else { -60.0 };
                        
                        egui::Frame::NONE
                            .fill(Color32::from_black_alpha(80))
                            .corner_radius(2.0)
                            .show(ui, |ui| {
                                ui.set_min_width(CHANNEL_WIDTH);
                                ui.centered_and_justified(|ui| {
                                    ui.label(RichText::new(format!("{:.1}", db)).size(10.0).monospace());
                                });
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
            .fill(ui.visuals().window_fill.linear_multiply(0.8)) // Darker for master
            .stroke(Stroke::new(1.0, ui.visuals().window_stroke().color))
            .inner_margin(4.0)
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.allocate_ui(Vec2::new(CHANNEL_WIDTH + 14.0, available_height), |ui| {
                    ui.vertical(|ui| {
                        // Title
                         ui.label(
                            RichText::new("MASTER")
                                .size(12.0)
                                .color(Color32::from_rgb(200, 180, 255))
                                .strong()
                        );
                        
                        ui.add_space(28.0 + 8.0); // Align with channels (rough approx of space taken by M/S)
                        
                        // Pan place holder
                        ui.label(RichText::new("PAN").size(9.0).color(ui.visuals().weak_text_color()));
                        ui.add_enabled(false, egui::Slider::new(&mut 0.0f32, -1.0..=1.0).show_value(false));
                        ui.label(RichText::new("C").size(9.0).color(ui.visuals().weak_text_color()));
                        
                        ui.add_space(8.0);
                        
                        // Fader + Meter
                        let remaining = (ui.available_height() - 50.0).max(10.0);
                        
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
                                if fader_response.double_clicked() {
                                     send_cmd(Command::SetMasterVolume { volume: 1.0 });
                                }
                            });
                        });
                        
                        // dB display
                        let db = if state.master_volume > 0.0 { 20.0 * (state.master_volume as f64).log10() } else { -60.0 };
                        
                        egui::Frame::NONE
                            .fill(Color32::from_black_alpha(80))
                            .corner_radius(2.0)
                            .show(ui, |ui| {
                                ui.set_min_width(CHANNEL_WIDTH);
                                ui.centered_and_justified(|ui| {
                                    ui.label(RichText::new(format!("{:.1}", db)).size(10.0).monospace());
                                });
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
        painter.rect_filled(rect, 2.0, Color32::from_black_alpha(150));
        
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
        // Background
        painter.rect_filled(rect, 1.0, Color32::from_black_alpha(200));
        
        // Convert level to dB
        let db = if level > 0.0 {
            20.0 * (level as f64).log10()
        } else {
            -60.0
        };
        
        // Constants for metering
        const SEGMENT_HEIGHT: f32 = 2.0;
        const GAP: f32 = 1.0;
        let total_steps = (rect.height() / (SEGMENT_HEIGHT + GAP)) as i32;
        
        for i in 0..total_steps {
            let y_pos = rect.max.y - (i as f32 * (SEGMENT_HEIGHT + GAP)) - SEGMENT_HEIGHT;
            if y_pos < rect.min.y { break; }
            
            let segment_rect = Rect::from_min_max(
                Pos2::new(rect.min.x, y_pos),
                Pos2::new(rect.max.x, y_pos + SEGMENT_HEIGHT)
            );
            
            // Calculate dB for this step
            // Map 0..total_steps to -60..+6 dB
            let step_normalized = i as f64 / total_steps as f64;
            let step_db = -60.0 + (step_normalized * 66.0);
            
            // Determine color based on step dB (not current level)
            let mut color = if step_db > 0.0 {
                DawColors::METER_RED
            } else if step_db > -12.0 {
                DawColors::METER_YELLOW
            } else {
                DawColors::METER_GREEN
            };
            
            // Dim if signal is below this step
            if db < step_db {
                color = color.linear_multiply(0.2);
            }
            
            painter.rect_filled(segment_rect, 1.0, color);
        }
    }
}

//! Track List Panel
//!
//! Track headers with name, volume, pan, mute, solo controls

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot, TrackState};
use crate::theme::DawColors;

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
        
        // Track list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0; // Connect tracks seamlessly
                for (i, track) in state.tracks.iter().enumerate() {
                    self.draw_track_header(ui, track, i, &mut send_cmd);
                }
            });
    }
    
    fn draw_track_header(
        &mut self,
        ui: &mut egui::Ui,
        track: &TrackState,
        index: usize,
        send_cmd: &mut impl FnMut(Command),
    ) {
        let available_width = ui.available_width();
        
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, TRACK_HEIGHT),
            Sense::click()
        );
        
        // Background
        let is_selected = track.is_selected;
        let bg_color = if is_selected {
            ui.visuals().selection.bg_fill.linear_multiply(0.3)
        } else {
            // Zebra striping
            if index % 2 == 0 {
                ui.visuals().faint_bg_color
            } else {
                ui.visuals().window_fill
            }
        };
        
        ui.painter().rect_filled(rect, 0.0, bg_color);
        
        // Bottom Border
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, ui.visuals().window_stroke().color)
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
        ui.painter().rect_filled(color_bar, 0.0, color);
        
        // Content area
        let content_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + 12.0, rect.min.y + 6.0),
            Pos2::new(rect.max.x - 4.0, rect.max.y - 4.0)
        );
        
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            ui.vertical(|ui| {
                // Top row: Name + Type
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
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
                                    .color(ui.visuals().text_color())
                                    .strong()
                            ).sense(Sense::click())
                        );
                        
                        if name_response.double_clicked() {
                            self.renaming_track = Some(track.id);
                            self.rename_buffer = track.name.clone();
                        }
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                         // Track type indicator
                        let type_text = if track.is_midi { "MIDI" } else { "AUD" };
                        ui.label(
                            RichText::new(type_text)
                                .size(9.0)
                                .color(ui.visuals().weak_text_color())
                        );
                    });
                });
                
                ui.add_space(4.0);
                
                // Mute / Solo buttons
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let mute_color = if track.is_muted {
                        DawColors::METER_RED
                    } else {
                        ui.visuals().weak_text_color()
                    };
                    
                    if ui.add(
                        egui::Button::new(RichText::new("M").size(11.0).color(mute_color).strong())
                            .min_size(Vec2::splat(20.0))
                            .frame(true)
                    ).clicked() {
                        send_cmd(Command::SetTrackMute { track_id: track.id, muted: !track.is_muted });
                    }
                    
                    let solo_color = if track.is_soloed {
                        DawColors::METER_YELLOW
                    } else {
                        ui.visuals().weak_text_color()
                    };
                    
                    if ui.add(
                        egui::Button::new(RichText::new("S").size(11.0).color(solo_color).strong())
                            .min_size(Vec2::splat(20.0))
                            .frame(true)
                    ).clicked() {
                        send_cmd(Command::SetTrackSolo { track_id: track.id, soloed: !track.is_soloed });
                    }
                    
                    if track.is_midi {
                        let rec_color = if track.is_armed {
                            DawColors::RECORD
                        } else {
                            ui.visuals().weak_text_color()
                        };
                        
                        if ui.add(
                            egui::Button::new(RichText::new("R").size(11.0).color(rec_color).strong())
                                .min_size(Vec2::splat(20.0))
                                .frame(true)
                        ).clicked() {
                            send_cmd(Command::SetTrackArm { track_id: track.id, armed: !track.is_armed });
                        }
                    }
                });
                
                ui.add_space(4.0);
                
                // Volume/Pan (simplified for header)
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Vol").size(9.0).color(ui.visuals().weak_text_color()));
                    let mut vol = track.volume;
                    if ui.add(
                        egui::Slider::new(&mut vol, 0.0..=1.5)
                            .show_value(false)
                    ).changed() {
                        send_cmd(Command::SetTrackVolume { track_id: track.id, volume: vol });
                    };
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

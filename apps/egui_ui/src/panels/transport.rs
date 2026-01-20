//! Transport Panel
//!
//! Play/Pause/Stop, Record, Loop, BPM, Time Display

use eframe::egui::{self, Color32, RichText, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot};
use crate::theme::DawColors;

#[derive(Default)]
pub struct TransportPanel {
    bpm_edit: Option<String>,
}

impl TransportPanel {
    pub fn show(
        &mut self, 
        ui: &mut egui::Ui, 
        state: &StateSnapshot,
        mut send_cmd: impl FnMut(Command),
    ) {
        // Center the transport bar
        ui.vertical_centered(|ui| {
            ui.set_min_height(60.0); // Ensure minimum height for the bar
            
            ui.horizontal(|ui| {
                 // Spacing from left
                let total_width = ui.available_width();
                // A very rough attempt to center:
                // We use spaces. But better is to just lay it out.
                // Since vertical_centered centers the block, horizontal just flows.
                // To truly center a horizontal strip, we need layout details.
                // Let's just use nice spacing.
                
                ui.add_space((total_width - 800.0).max(0.0) / 2.0);
            
                ui.spacing_mut().item_spacing.x = 16.0; // Comfort spacing
                
                // === Transport Buttons Group ===
                ui.group(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.style_mut().visuals.widgets.inactive.corner_radius = eframe::egui::CornerRadius::same(18);
                    
                    // Rewind to start
                    if transport_button(ui, "⏮", "Return to Start", false).clicked() {
                        send_cmd(Command::SetPlayhead { beats: 0.0 });
                    }
                    
                    // Stop
                    if transport_button(ui, "⏹", "Stop", !state.transport.is_playing && state.transport.playhead_beats == 0.0).clicked() {
                        send_cmd(Command::Stop);
                    }
                    
                    // Play/Pause
                    let play_icon = if state.transport.is_playing { "⏸" } else { "▶" };
                    let play_tip = if state.transport.is_playing { "Pause" } else { "Play" };
                    
                    let play_is_active = state.transport.is_playing;
                    let play_color = if play_is_active { Some(DawColors::PLAY) } else { None };
                    
                    if transport_button_custom(ui, play_icon, play_tip, play_is_active, play_color).clicked() {
                         if state.transport.is_playing { send_cmd(Command::Pause); } else { send_cmd(Command::Play); }
                    }
                    
                    // Record
                    if transport_button_colored(ui, "⏺", "Record", state.transport.is_recording, DawColors::RECORD).clicked() {
                        send_cmd(Command::ToggleRecord);
                    }
                    
                    // Loop Toggle
                    let loop_active = state.transport.is_looping;
                    if transport_button_colored(ui, "🔁", "Toggle Loop", loop_active, DawColors::LOOP).clicked() {
                         send_cmd(Command::ToggleLoop);
                    }
                });
                
                ui.add_space(8.0);
                
                // === Time Display (LCD) ===
                let position_str = format_position(
                    state.transport.playhead_beats,
                    state.transport.time_sig_num,
                );
                
                egui::Frame::NONE
                    .fill(Color32::from_rgb(5, 5, 8))
                    .stroke(egui::Stroke::new(2.0, Color32::from_rgb(30, 30, 35)))
                    .inner_margin(egui::Margin::symmetric(16, 6))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&position_str)
                                .size(32.0)
                                .monospace()
                                .color(DawColors::METER_GREEN.linear_multiply(1.2))
                        ).on_hover_text("Position: Bar.Beat.Tick");
                    });
                
                ui.add_space(8.0);
                
                // === Info Group ===
                ui.group(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    
                     // BPM
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("BPM").size(10.0).color(ui.visuals().weak_text_color()));
                        // Editable BPM field
                        let bpm_text = self.bpm_edit.clone()
                            .unwrap_or_else(|| format!("{:.1}", state.transport.tempo));
                        
                        let mut edit_text = self.bpm_edit.get_or_insert(bpm_text).clone();
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut edit_text)
                                .desired_width(50.0)
                                .font(egui::TextStyle::Monospace)
                        );
                        self.bpm_edit = Some(edit_text);
                        
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Some(ref text) = self.bpm_edit {
                                if let Ok(new_bpm) = text.parse::<f64>() {
                                    send_cmd(Command::SetTempo { bpm: new_bpm });
                                }
                            }
                            self.bpm_edit = None;
                        }
                    });

                     // Time Sig
                    ui.separator();
                     ui.horizontal(|ui| {
                        ui.label(RichText::new("SIG").size(10.0).color(ui.visuals().weak_text_color()));
                        ui.label(format!("{}/{}", state.transport.time_sig_num, state.transport.time_sig_denom));
                    });
                     
                    // CPU/Time
                    ui.separator();
                     ui.horizontal(|ui| {
                        let seconds = state.transport.playhead_seconds;
                        let minutes = (seconds / 60.0) as i32;
                        let secs = seconds % 60.0;
                        ui.monospace(format!("{:02}:{:05.2}", minutes, secs));
                    });
                });
            });
        });
    }
}

fn transport_button(ui: &mut egui::Ui, icon: &str, tooltip: &str, active: bool) -> egui::Response {
    let color = if active {
        DawColors::PLAY // Default active color
    } else {
        ui.visuals().text_color()
    };
    
    ui.add(
        egui::Button::new(RichText::new(icon).size(22.0).color(color))
            .min_size(Vec2::splat(36.0))
            .frame(true)
    ).on_hover_text(tooltip)
}

fn transport_button_colored(
    ui: &mut egui::Ui, 
    icon: &str, 
    tooltip: &str, 
    active: bool,
    active_color: Color32
) -> egui::Response {
    let color = if active { active_color } else { ui.visuals().weak_text_color() };
    
    ui.add(
        egui::Button::new(RichText::new(icon).size(22.0).color(color))
            .min_size(Vec2::splat(36.0))
            .frame(true)
    ).on_hover_text(tooltip)
}

fn transport_button_custom(
    ui: &mut egui::Ui, 
    icon: &str, 
    tooltip: &str, 
    active: bool,
    active_color_opt: Option<Color32>
) -> egui::Response {
    let mut text = RichText::new(icon).size(22.0);
    
    if active {
        if let Some(col) = active_color_opt {
            text = text.color(col);
        }
    } else {
        text = text.color(ui.visuals().text_color());
    }
    
    ui.add(
        egui::Button::new(text)
            .min_size(Vec2::splat(36.0))
            .frame(true)
            .fill(if active { ui.visuals().selection.bg_fill } else { Color32::TRANSPARENT })
    ).on_hover_text(tooltip)
}

fn format_position(beats: f64, beats_per_bar: u32) -> String {
    let beats_per_bar = beats_per_bar.max(1) as f64;
    let bar = (beats / beats_per_bar) as i32 + 1;
    let beat_in_bar = (beats % beats_per_bar) + 1.0;
    let whole_beat = beat_in_bar as i32;
    // Standard DAW style: Bar.Beat.Percent/Tick
    let ticks = ((beat_in_bar - whole_beat as f64) * 100.0) as i32;
    
    format!("{:03}.{:02}.{:02}", bar, whole_beat, ticks)
}

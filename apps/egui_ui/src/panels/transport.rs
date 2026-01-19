//! Transport Panel
//!
//! Play/Pause/Stop, Record, Loop, BPM, Time Display

use eframe::egui::{self, Color32, RichText, Sense, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot};

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
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            
            // === Transport Buttons ===
            ui.horizontal(|ui| {
                // Rewind to start
                if transport_button(ui, "⏮", "Return to Start", false).clicked() {
                    send_cmd(Command::SetPlayhead { beats: 0.0 });
                }
                
                // Stop
                if transport_button(ui, "⏹", "Stop", false).clicked() {
                    send_cmd(Command::Stop);
                }
                
                // Play/Pause
                let play_icon = if state.transport.is_playing { "⏸" } else { "▶" };
                let play_tip = if state.transport.is_playing { "Pause" } else { "Play" };
                if transport_button(ui, play_icon, play_tip, state.transport.is_playing).clicked() {
                    if state.transport.is_playing {
                        send_cmd(Command::Pause);
                    } else {
                        send_cmd(Command::Play);
                    }
                }
                
                // Record
                if transport_button_colored(
                    ui, 
                    "⏺", 
                    "Record", 
                    state.transport.is_recording,
                    Color32::from_rgb(220, 50, 50)
                ).clicked() {
                    send_cmd(Command::ToggleRecord);
                }
            });
            
            ui.separator();
            
            // === Loop Toggle ===
            let loop_color = if state.transport.is_looping {
                Color32::from_rgb(100, 180, 255)
            } else {
                Color32::GRAY
            };
            if ui.add(
                egui::Button::new(RichText::new("🔁").size(18.0).color(loop_color))
                    .frame(false)
            ).on_hover_text("Toggle Loop").clicked() {
                send_cmd(Command::ToggleLoop);
            }
            
            ui.separator();
            
            // === Time Display ===
            let position_str = format_position(
                state.transport.playhead_beats,
                state.transport.time_sig_num,
            );
            ui.label(
                RichText::new(&position_str)
                    .size(24.0)
                    .monospace()
                    .color(Color32::from_rgb(0, 255, 100))
            );
            
            ui.separator();
            
            // === BPM ===
            ui.label("BPM:");
            
            // Editable BPM field
            let bpm_text = self.bpm_edit.clone()
                .unwrap_or_else(|| format!("{:.1}", state.transport.bpm));
            
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.bpm_edit.get_or_insert(bpm_text))
                    .desired_width(60.0)
                    .font(egui::TextStyle::Monospace)
            );
            
            if response.lost_focus() {
                if let Some(ref text) = self.bpm_edit {
                    if let Ok(new_bpm) = text.parse::<f64>() {
                        send_cmd(Command::SetBpm { bpm: new_bpm });
                    }
                }
                self.bpm_edit = None;
            }
            
            ui.separator();
            
            // === Time Signature ===
            ui.label(format!("{}/{}", 
                state.transport.time_sig_num, 
                state.transport.time_sig_denom
            ));
            
            // === Right side: Seconds display ===
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let seconds = state.transport.playhead_seconds;
                let minutes = (seconds / 60.0) as i32;
                let secs = seconds % 60.0;
                ui.label(
                    RichText::new(format!("{:02}:{:05.2}", minutes, secs))
                        .size(18.0)
                        .monospace()
                        .color(Color32::GRAY)
                );
            });
        });
    }
}

fn transport_button(ui: &mut egui::Ui, icon: &str, tooltip: &str, active: bool) -> egui::Response {
    let color = if active {
        Color32::from_rgb(100, 200, 100)
    } else {
        Color32::WHITE
    };
    
    ui.add(
        egui::Button::new(RichText::new(icon).size(24.0).color(color))
            .min_size(Vec2::splat(40.0))
    ).on_hover_text(tooltip)
}

fn transport_button_colored(
    ui: &mut egui::Ui, 
    icon: &str, 
    tooltip: &str, 
    active: bool,
    active_color: Color32
) -> egui::Response {
    let color = if active { active_color } else { Color32::GRAY };
    
    ui.add(
        egui::Button::new(RichText::new(icon).size(24.0).color(color))
            .min_size(Vec2::splat(40.0))
    ).on_hover_text(tooltip)
}

fn format_position(beats: f64, beats_per_bar: u8) -> String {
    let beats_per_bar = beats_per_bar.max(1) as f64;
    let bar = (beats / beats_per_bar) as i32 + 1;
    let beat_in_bar = (beats % beats_per_bar) + 1.0;
    let whole_beat = beat_in_bar as i32;
    let ticks = ((beat_in_bar - whole_beat as f64) * 960.0) as i32;
    
    format!("{:03}.{}.{:03}", bar, whole_beat, ticks)
}

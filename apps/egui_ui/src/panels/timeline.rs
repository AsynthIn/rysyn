//! Timeline Panel
//!
//! Main arrangement view with clips, playhead, grid

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot, ClipState};

const PIXELS_PER_BEAT: f32 = 40.0;
const TRACK_HEIGHT: f32 = 80.0;
const RULER_HEIGHT: f32 = 24.0;

#[derive(Default)]
pub struct TimelinePanel {
    dragging_clip: Option<u32>,
    drag_start: Option<Pos2>,
}

impl TimelinePanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        mut send_cmd: impl FnMut(Command),
        zoom: &mut f32,
        scroll: &mut f64,
    ) {
        let available = ui.available_rect_before_wrap();
        let ppb = PIXELS_PER_BEAT * *zoom;
        
        // === Ruler ===
        let ruler_rect = Rect::from_min_size(
            available.min,
            Vec2::new(available.width(), RULER_HEIGHT)
        );
        
        ui.allocate_ui_at_rect(ruler_rect, |ui| {
            self.draw_ruler(ui, state, ppb, *scroll);
        });
        
        // === Timeline Area ===
        let timeline_rect = Rect::from_min_max(
            Pos2::new(available.min.x, ruler_rect.max.y),
            available.max
        );
        
        let response = ui.allocate_rect(timeline_rect, Sense::click_and_drag());
        
        // Handle zoom with scroll wheel
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta);
            if ui.input(|i| i.modifiers.ctrl) {
                // Ctrl + scroll = zoom
                *zoom = (*zoom * (1.0 + scroll_delta.y * 0.001)).clamp(0.1, 10.0);
            } else {
                // Regular scroll = horizontal scroll
                *scroll = (*scroll - (scroll_delta.y as f64 / ppb as f64)).max(0.0);
            }
        }
        
        // Handle click to set playhead
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let beat = *scroll + ((pos.x - timeline_rect.min.x) / ppb) as f64;
                send_cmd(Command::SetPlayhead { beats: beat });
            }
        }
        
        let painter = ui.painter_at(timeline_rect);
        
        // Background
        painter.rect_filled(timeline_rect, 0.0, Color32::from_rgb(30, 30, 35));
        
        // Grid lines
        self.draw_grid(&painter, timeline_rect, state, ppb, *scroll);
        
        // Track lanes
        self.draw_track_lanes(&painter, timeline_rect, state, ppb, *scroll);
        
        // Clips
        for clip in &state.clips {
            self.draw_clip(&painter, timeline_rect, clip, state, ppb, *scroll);
        }
        
        // Playhead
        let playhead_x = timeline_rect.min.x + 
            ((state.transport.playhead_beats - *scroll) * ppb as f64) as f32;
        
        if playhead_x >= timeline_rect.min.x && playhead_x <= timeline_rect.max.x {
            painter.line_segment(
                [Pos2::new(playhead_x, timeline_rect.min.y), 
                 Pos2::new(playhead_x, timeline_rect.max.y)],
                Stroke::new(2.0, Color32::from_rgb(255, 100, 100))
            );
            
            // Playhead head
            let head_size = 8.0;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(playhead_x, timeline_rect.min.y),
                    Pos2::new(playhead_x - head_size, timeline_rect.min.y - head_size),
                    Pos2::new(playhead_x + head_size, timeline_rect.min.y - head_size),
                ],
                Color32::from_rgb(255, 100, 100),
                Stroke::NONE
            ));
        }
        
        // Loop region overlay
        if state.transport.is_looping {
            let loop_start_x = timeline_rect.min.x + 
                ((state.transport.loop_start_beats - *scroll) * ppb as f64) as f32;
            let loop_end_x = timeline_rect.min.x + 
                ((state.transport.loop_end_beats - *scroll) * ppb as f64) as f32;
            
            if loop_end_x > timeline_rect.min.x && loop_start_x < timeline_rect.max.x {
                let loop_rect = Rect::from_min_max(
                    Pos2::new(loop_start_x.max(timeline_rect.min.x), timeline_rect.min.y),
                    Pos2::new(loop_end_x.min(timeline_rect.max.x), timeline_rect.max.y)
                );
                painter.rect_filled(loop_rect, 0.0, Color32::from_rgba_unmultiplied(100, 180, 255, 30));
            }
        }
    }
    
    fn draw_ruler(
        &self, 
        ui: &mut egui::Ui, 
        state: &StateSnapshot, 
        ppb: f32,
        scroll: f64
    ) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(45, 45, 50));
        
        let beats_per_bar = state.transport.time_sig_num.max(1) as f64;
        let start_beat = scroll.floor();
        let end_beat = scroll + (rect.width() / ppb) as f64 + 1.0;
        
        let mut beat = start_beat;
        while beat < end_beat {
            let x = rect.min.x + ((beat - scroll) * ppb as f64) as f32;
            
            let is_bar = (beat % beats_per_bar).abs() < 0.01;
            
            if is_bar {
                // Bar line
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(120, 120, 120))
                );
                
                // Bar number
                let bar_num = (beat / beats_per_bar) as i32 + 1;
                painter.text(
                    Pos2::new(x + 4.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    format!("{}", bar_num),
                    egui::FontId::proportional(11.0),
                    Color32::WHITE
                );
            } else {
                // Beat tick
                painter.line_segment(
                    [Pos2::new(x, rect.max.y - 6.0), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                );
            }
            
            beat += 1.0;
        }
    }
    
    fn draw_grid(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        state: &StateSnapshot,
        ppb: f32,
        scroll: f64,
    ) {
        let beats_per_bar = state.transport.time_sig_num.max(1) as f64;
        let start_beat = scroll.floor();
        let end_beat = scroll + (rect.width() / ppb) as f64 + 1.0;
        
        let mut beat = start_beat;
        while beat < end_beat {
            let x = rect.min.x + ((beat - scroll) * ppb as f64) as f32;
            
            let is_bar = (beat % beats_per_bar).abs() < 0.01;
            let color = if is_bar {
                Color32::from_rgb(60, 60, 65)
            } else {
                Color32::from_rgb(40, 40, 45)
            };
            
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, color)
            );
            
            beat += 1.0;
        }
    }
    
    fn draw_track_lanes(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        state: &StateSnapshot,
        _ppb: f32,
        _scroll: f64,
    ) {
        for (i, track) in state.tracks.iter().enumerate() {
            let y = rect.min.y + (i as f32 * TRACK_HEIGHT);
            
            if y > rect.max.y {
                break;
            }
            
            // Track lane background
            let lane_color = if i % 2 == 0 {
                Color32::from_rgb(35, 35, 40)
            } else {
                Color32::from_rgb(32, 32, 37)
            };
            
            let lane_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, y),
                Vec2::new(rect.width(), TRACK_HEIGHT)
            );
            painter.rect_filled(lane_rect, 0.0, lane_color);
            
            // Track separator
            painter.line_segment(
                [Pos2::new(rect.min.x, y + TRACK_HEIGHT), 
                 Pos2::new(rect.max.x, y + TRACK_HEIGHT)],
                Stroke::new(1.0, Color32::from_rgb(50, 50, 55))
            );
        }
    }
    
    fn draw_clip(
        &self,
        painter: &egui::Painter,
        timeline_rect: Rect,
        clip: &ClipState,
        state: &StateSnapshot,
        ppb: f32,
        scroll: f64,
    ) {
        // Find track index
        let track_index = state.tracks.iter()
            .position(|t| t.id == clip.track_id)
            .unwrap_or(0);
        
        let y = timeline_rect.min.y + (track_index as f32 * TRACK_HEIGHT) + 4.0;
        let x = timeline_rect.min.x + ((clip.start_beats - scroll) * ppb as f64) as f32;
        let width = (clip.length_beats * ppb as f64) as f32;
        
        if x + width < timeline_rect.min.x || x > timeline_rect.max.x {
            return; // Off screen
        }
        
        let clip_rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(width, TRACK_HEIGHT - 8.0)
        );
        
        // Clip color from state or default
        let color = Color32::from_rgba_unmultiplied(
            ((clip.color >> 24) & 0xFF) as u8,
            ((clip.color >> 16) & 0xFF) as u8,
            ((clip.color >> 8) & 0xFF) as u8,
            220
        );
        
        // Selection highlight
        if clip.is_selected {
            painter.rect_stroke(
                clip_rect.expand(2.0),
                4.0,
                Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Outside
            );
        }
        
        // Clip body
        painter.rect_filled(clip_rect, 4.0, color);
        
        // Clip border
        painter.rect_stroke(
            clip_rect,
            4.0,
            Stroke::new(1.0, color.linear_multiply(0.7)),
            egui::StrokeKind::Outside
        );
        
        // Clip name
        let text_rect = clip_rect.shrink(4.0);
        painter.text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            &clip.name,
            egui::FontId::proportional(11.0),
            Color32::WHITE
        );
        
        // MIDI/Audio indicator
        let indicator = if clip.is_midi { "♪" } else { "🎵" };
        painter.text(
            Pos2::new(text_rect.right() - 16.0, text_rect.top()),
            egui::Align2::LEFT_TOP,
            indicator,
            egui::FontId::proportional(10.0),
            Color32::WHITE.linear_multiply(0.7)
        );
    }
}

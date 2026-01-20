//! Timeline Panel
//!
//! Main arrangement view with clips, playhead, grid

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, CornerRadius};
use rysyn_ffi_bridge::{Command, StateSnapshot, ClipState};
use crate::theme::{DawColors, ColorExt};

const PIXELS_PER_BEAT: f32 = 40.0;
const TRACK_HEIGHT: f32 = 80.0;
const RULER_HEIGHT: f32 = 28.0;

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
        
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(ruler_rect), |ui| {
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
        painter.rect_filled(timeline_rect, 0.0, ui.visuals().panel_fill);
        
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
                Stroke::new(2.0, DawColors::PLAYHEAD)
            );
            
            // Playhead head
            let head_size = 10.0;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(playhead_x, timeline_rect.min.y),
                    Pos2::new(playhead_x - head_size, timeline_rect.min.y - head_size),
                    Pos2::new(playhead_x + head_size, timeline_rect.min.y - head_size),
                ],
                DawColors::PLAYHEAD,
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
                painter.rect_filled(loop_rect, 0.0, DawColors::LOOP.linear_multiply(0.15));
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
        
        // Ruler Background gradient
        let bg_gradient = egui::Mesh::with_texture(egui::TextureId::default());
        // Simple dark bg
        painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 35));
        
        // Bottom border
        painter.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, Color32::from_rgb(60, 60, 65))
        );
        
        let beats_per_bar = state.transport.time_sig_num.max(1) as f64;
        let start_beat = scroll.floor();
        let end_beat = scroll + (rect.width() / ppb) as f64 + 1.0;
        
        let mut beat = start_beat;
        while beat < end_beat {
            let x = rect.min.x + ((beat - scroll) * ppb as f64) as f32;
            
            let is_bar = (beat % beats_per_bar).abs() < 0.01;
            
            if is_bar {
                // Bar tick (Major)
                painter.line_segment(
                    [Pos2::new(x, rect.max.y - 12.0), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(200, 200, 200))
                );
                
                // Bar number
                let bar_num = (beat / beats_per_bar) as i32 + 1;
                painter.text(
                    Pos2::new(x + 4.0, rect.max.y - 18.0),
                    egui::Align2::LEFT_CENTER,
                    format!("{}", bar_num),
                    egui::FontId::monospace(10.0),
                    Color32::from_rgb(180, 180, 180)
                );
            } else {
                // Beat tick (Minor)
                painter.line_segment(
                    [Pos2::new(x, rect.max.y - 6.0), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(100, 100, 100))
                );
            }
            
            beat += 1.0;
        }
        
        // Playhead indicator in ruler
        let playhead_x = rect.min.x + ((state.transport.playhead_beats - scroll) * ppb as f64) as f32;
        if playhead_x >= rect.min.x && playhead_x <= rect.max.x {
            // Triangle pointer
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(playhead_x - 6.0, rect.max.y - 8.0),
                    Pos2::new(playhead_x + 6.0, rect.max.y - 8.0),
                    Pos2::new(playhead_x, rect.max.y),
                ],
                DawColors::PLAYHEAD,
                Stroke::NONE
            ));
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
        
        // Draw vertical grid lines
        let mut beat = start_beat;
        while beat < end_beat {
            let x = rect.min.x + ((beat - scroll) * ppb as f64) as f32;
            
            let is_bar = (beat % beats_per_bar).abs() < 0.01;
            
            if is_bar {
                // Highlight bar areas (alternate slightly)
                let bar_idx = (beat / beats_per_bar) as i64;
                if bar_idx % 2 == 0 {
                   let next_x = rect.min.x + ((beat + beats_per_bar - scroll) * ppb as f64) as f32;
                   let bg_rect = Rect::from_min_max(
                       Pos2::new(x, rect.min.y),
                       Pos2::new(next_x, rect.max.y)
                   );
                   painter.rect_filled(bg_rect, 0.0, Color32::from_white_alpha(3));
                }
                
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, DawColors::GRID_BAR.linear_multiply(0.5))
                );
            } else {
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, DawColors::GRID_BEAT.linear_multiply(0.3))
                );
            }
            
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
        for (i, _track) in state.tracks.iter().enumerate() {
            let y = rect.min.y + (i as f32 * TRACK_HEIGHT);
            
            if y > rect.max.y {
                break;
            }
            
            // Track lane background (zebra striping)
            let lane_color = if i % 2 == 0 {
                Color32::from_white_alpha(5)
            } else {
                Color32::TRANSPARENT
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
                Stroke::new(1.0, DawColors::GRID_BEAT)
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
        
        let y = timeline_rect.min.y + (track_index as f32 * TRACK_HEIGHT) + 2.0;
        let x = timeline_rect.min.x + ((clip.start_beats - scroll) * ppb as f64) as f32;
        let width = (clip.length_beats * ppb as f64) as f32;
        
        if x + width < timeline_rect.min.x || x > timeline_rect.max.x {
            return; // Off screen
        }
        
        let clip_rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(width, TRACK_HEIGHT - 4.0)
        );
        
        // Clip color from state or default
        let base_color = Color32::from_rgba_unmultiplied(
            ((clip.color >> 24) & 0xFF) as u8,
            ((clip.color >> 16) & 0xFF) as u8,
            ((clip.color >> 8) & 0xFF) as u8,
            255
        );
        
        // Selection highlight
        let stroke = if clip.is_selected {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.0, base_color.lighten())
        };
        
        // Clip body with gradient-like look (solid for now)
        painter.rect_filled(clip_rect, 4.0, base_color);
        
        // Clip border
        painter.rect_stroke(
            clip_rect,
            4.0,
            stroke,
            egui::StrokeKind::Inside
        );
        
        // Clip header background
        let header_rect = Rect::from_min_size(
            clip_rect.min, 
            Vec2::new(clip_rect.width(), 16.0)
        );
        painter.rect_filled(
            header_rect, 
            CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 }, 
            Color32::from_black_alpha(40)
        );
        
        // Clip name
        let text_rect = header_rect.shrink(2.0);
        painter.text(
            text_rect.left_center(),
            egui::Align2::LEFT_CENTER,
            &clip.name,
            egui::FontId::proportional(11.0),
            Color32::WHITE
        );
        
        // MIDI/Audio indicator
        let indicator = if clip.is_midi { "♪" } else { "🎵" };
        painter.text(
            Pos2::new(text_rect.right() - 4.0, text_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            indicator,
            egui::FontId::proportional(11.0),
            Color32::WHITE.linear_multiply(0.8)
        );
    }
}

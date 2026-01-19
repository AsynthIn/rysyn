//! Knob Widget
//!
//! Rotary control for volume, pan, etc.

use eframe::egui::{self, Color32, Response, Sense, Stroke, Ui, Vec2};
use std::f32::consts::PI;

/// A rotary knob control
pub struct Knob<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    size: f32,
    label: Option<&'a str>,
}

impl<'a> Knob<'a> {
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            size: 40.0,
            label: None,
        }
    }
    
    pub fn range(mut self, range: std::ops::RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }
    
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl<'a> egui::Widget for Knob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::splat(self.size);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        
        if response.dragged() {
            let delta = response.drag_delta();
            let sensitivity = 0.005;
            let range_size = *self.range.end() - *self.range.start();
            *self.value += -delta.y * sensitivity * range_size;
            *self.value = self.value.clamp(*self.range.start(), *self.range.end());
        }
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let radius = rect.width() / 2.0 - 4.0;
            
            // Background circle
            painter.circle_filled(center, radius, Color32::from_rgb(40, 40, 45));
            painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::from_rgb(60, 60, 65)));
            
            // Value arc
            let normalized = (*self.value - *self.range.start()) / (*self.range.end() - *self.range.start());
            let start_angle = PI * 0.75; // 135 degrees
            let end_angle = PI * 2.25;   // 405 degrees
            let current_angle = start_angle + normalized * (end_angle - start_angle);
            
            // Draw arc
            let arc_radius = radius - 4.0;
            let n_points = 32;
            for i in 0..n_points {
                let t = i as f32 / n_points as f32;
                let angle = start_angle + t * (current_angle - start_angle);
                if angle > current_angle {
                    break;
                }
                let next_t = (i + 1) as f32 / n_points as f32;
                let next_angle = (start_angle + next_t * (current_angle - start_angle)).min(current_angle);
                
                let p1 = center + Vec2::new(angle.cos(), angle.sin()) * arc_radius;
                let p2 = center + Vec2::new(next_angle.cos(), next_angle.sin()) * arc_radius;
                
                painter.line_segment([p1, p2], Stroke::new(3.0, Color32::from_rgb(100, 180, 255)));
            }
            
            // Pointer
            let pointer_length = radius - 8.0;
            let pointer_end = center + Vec2::new(current_angle.cos(), current_angle.sin()) * pointer_length;
            painter.line_segment(
                [center, pointer_end],
                Stroke::new(2.0, Color32::WHITE)
            );
            
            // Center dot
            painter.circle_filled(center, 3.0, Color32::WHITE);
        }
        
        response
    }
}

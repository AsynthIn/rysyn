//! VU Meter Widget
//!
//! Audio level visualization

use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// A vertical VU meter
pub struct VuMeter {
    level: f32,      // 0.0 to 1.0
    peak: f32,       // Peak hold
    width: f32,
    height: f32,
}

impl VuMeter {
    pub fn new(level: f32) -> Self {
        Self {
            level,
            peak: level,
            width: 12.0,
            height: 100.0,
        }
    }
    
    pub fn with_peak(mut self, peak: f32) -> Self {
        self.peak = peak;
        self
    }
    
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl egui::Widget for VuMeter {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(self.width, self.height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Background
            painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 20, 25));
            
            // Meter segments
            let segments = 20;
            let segment_height = (self.height - 2.0) / segments as f32;
            let gap = 1.0;
            
            for i in 0..segments {
                let segment_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + 1.0,
                        rect.max.y - (i + 1) as f32 * segment_height - 1.0
                    ),
                    Vec2::new(self.width - 2.0, segment_height - gap)
                );
                
                let segment_threshold = i as f32 / segments as f32;
                
                let color = if segment_threshold > 0.9 {
                    Color32::from_rgb(255, 50, 50)  // Red zone
                } else if segment_threshold > 0.75 {
                    Color32::from_rgb(255, 200, 50) // Yellow zone
                } else {
                    Color32::from_rgb(50, 200, 100) // Green zone
                };
                
                let fill_color = if self.level > segment_threshold {
                    color
                } else {
                    color.linear_multiply(0.2)
                };
                
                painter.rect_filled(segment_rect, 0.0, fill_color);
            }
            
            // Peak indicator
            if self.peak > 0.0 {
                let peak_y = rect.max.y - (self.peak * (self.height - 2.0)) - 1.0;
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + 1.0, peak_y),
                        Pos2::new(rect.max.x - 1.0, peak_y)
                    ],
                    Stroke::new(2.0, Color32::WHITE)
                );
            }
        }
        
        response
    }
}

/// A stereo VU meter pair
pub struct StereoMeter {
    left: f32,
    right: f32,
    peak_left: f32,
    peak_right: f32,
    width: f32,
    height: f32,
}

impl StereoMeter {
    pub fn new(left: f32, right: f32) -> Self {
        Self {
            left,
            right,
            peak_left: left,
            peak_right: right,
            width: 28.0,
            height: 100.0,
        }
    }
    
    pub fn with_peaks(mut self, peak_left: f32, peak_right: f32) -> Self {
        self.peak_left = peak_left;
        self.peak_right = peak_right;
        self
    }
}

impl egui::Widget for StereoMeter {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(self.width, self.height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let meter_width = (self.width - 4.0) / 2.0;
            
            // Left meter
            let left_rect = Rect::from_min_size(
                rect.min,
                Vec2::new(meter_width, self.height)
            );
            
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
                ui.add(VuMeter::new(self.left)
                    .with_peak(self.peak_left)
                    .size(meter_width, self.height));
            });
            
            // Right meter
            let right_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + meter_width + 4.0, rect.min.y),
                Vec2::new(meter_width, self.height)
            );
            
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |ui| {
                ui.add(VuMeter::new(self.right)
                    .with_peak(self.peak_right)
                    .size(meter_width, self.height));
            });
        }
        
        response
    }
}

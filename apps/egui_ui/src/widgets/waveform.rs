//! Waveform Display Widget
//!
//! Audio waveform visualization for clips

use eframe::egui::{self, Color32, Pos2, Response, Sense, Stroke, Ui, Vec2};

/// Audio waveform display
pub struct Waveform<'a> {
    samples: &'a [f32],
    color: Color32,
    height: f32,
}

impl<'a> Waveform<'a> {
    pub fn new(samples: &'a [f32]) -> Self {
        Self {
            samples,
            color: Color32::from_rgb(100, 180, 255),
            height: 60.0,
        }
    }
    
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
    
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<'a> egui::Widget for Waveform<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let available_width = ui.available_width();
        let desired_size = Vec2::new(available_width, self.height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
        
        if ui.is_rect_visible(rect) && !self.samples.is_empty() {
            let painter = ui.painter();
            
            // Background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 35));
            
            let center_y = rect.center().y;
            let half_height = self.height / 2.0 - 2.0;
            
            // Center line
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, center_y),
                    Pos2::new(rect.max.x, center_y)
                ],
                Stroke::new(1.0, Color32::from_rgb(50, 50, 55))
            );
            
            // Calculate how many samples per pixel
            let samples_per_pixel = self.samples.len() as f32 / available_width;
            
            if samples_per_pixel < 1.0 {
                // Few samples: draw as line
                let points: Vec<Pos2> = self.samples.iter()
                    .enumerate()
                    .map(|(i, &sample)| {
                        let x = rect.min.x + (i as f32 / self.samples.len() as f32) * available_width;
                        let y = center_y - sample * half_height;
                        Pos2::new(x, y)
                    })
                    .collect();
                
                for window in points.windows(2) {
                    painter.line_segment([window[0], window[1]], Stroke::new(1.0, self.color));
                }
            } else {
                // Many samples: draw min/max envelope
                let pixels = available_width as usize;
                
                for px in 0..pixels {
                    let start_sample = (px as f32 * samples_per_pixel) as usize;
                    let end_sample = ((px + 1) as f32 * samples_per_pixel) as usize;
                    let end_sample = end_sample.min(self.samples.len());
                    
                    if start_sample >= self.samples.len() {
                        break;
                    }
                    
                    let chunk = &self.samples[start_sample..end_sample];
                    if chunk.is_empty() {
                        continue;
                    }
                    
                    let min = chunk.iter().cloned().fold(f32::INFINITY, f32::min);
                    let max = chunk.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    
                    let x = rect.min.x + px as f32;
                    let y_min = center_y - min * half_height;
                    let y_max = center_y - max * half_height;
                    
                    painter.line_segment(
                        [Pos2::new(x, y_min), Pos2::new(x, y_max)],
                        Stroke::new(1.0, self.color)
                    );
                }
            }
        }
        
        response
    }
}

/// Compact waveform overview (for clip thumbnails)
pub struct WaveformThumbnail<'a> {
    peaks: &'a [f32], // Pre-computed peaks
    color: Color32,
}

impl<'a> WaveformThumbnail<'a> {
    pub fn new(peaks: &'a [f32]) -> Self {
        Self {
            peaks,
            color: Color32::from_rgb(100, 180, 255),
        }
    }
    
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
}

impl<'a> egui::Widget for WaveformThumbnail<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::hover());
        
        if ui.is_rect_visible(rect) && !self.peaks.is_empty() {
            let painter = ui.painter();
            let center_y = rect.center().y;
            let half_height = rect.height() / 2.0 - 1.0;
            
            let width = rect.width();
            let peaks_per_pixel = self.peaks.len() as f32 / width;
            
            for px in 0..width as usize {
                let idx = (px as f32 * peaks_per_pixel) as usize;
                if idx >= self.peaks.len() {
                    break;
                }
                
                let peak = self.peaks[idx].abs();
                let x = rect.min.x + px as f32;
                
                painter.line_segment(
                    [
                        Pos2::new(x, center_y - peak * half_height),
                        Pos2::new(x, center_y + peak * half_height)
                    ],
                    Stroke::new(1.0, self.color.linear_multiply(0.8))
                );
            }
        }
        
        response
    }
}

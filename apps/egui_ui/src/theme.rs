//! Theme Configuration
//!
//! Custom fonts and dark theme for professional DAW look

use eframe::egui::{self, Color32, FontDefinitions, Style, Visuals, CornerRadius, Stroke, Margin};

/// Setup custom fonts for the DAW
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    
    // In a real app we would load a nice font like Inter or Roboto here.
    // For now we rely on the default egui font (Proportional) which is usually good.
    // We can tweak sizes.
    
    fonts.font_data.iter_mut().for_each(|(_name, font)| {
        // Handle Arc<FontData>
        std::sync::Arc::make_mut(font).tweak.scale = 1.05;
    });

    ctx.set_fonts(fonts);
}

/// Setup dark theme optimized for DAW workflow
pub fn setup_dark_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    
    // === Color Palette (Obsidian / Professional Dark) ===
    let bg_app      = Color32::from_rgb(18, 18, 20);      // Deepest background
    let bg_panel    = Color32::from_rgb(28, 28, 30);      // Panel background
    let bg_subpanel = Color32::from_rgb(34, 34, 37);      // Inner containers
    let bg_widget   = Color32::from_rgb(42, 42, 45);      // Default widget bg
    
    // Accents
    let accent_primary = Color32::from_rgb(50, 140, 240); // Professional Blue
    let accent_hover   = Color32::from_rgb(80, 160, 255);
    let accent_active  = Color32::from_rgb(30, 120, 220);
    
    // Text
    let text_primary   = Color32::from_rgb(240, 240, 240);
    let text_secondary = Color32::from_rgb(160, 160, 165);

    // Borders
    let border_color   = Color32::from_rgb(55, 55, 58);
    
    // === Spacing & Sizes ===
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = Margin::same(8);
    style.spacing.button_padding = egui::vec2(8.0, 5.0);
    style.spacing.menu_margin = Margin::same(6);
    style.spacing.indent = 18.0;
    
    // === Visuals ===
    let visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(text_primary),
        
        // Window/Panel
        panel_fill: bg_panel,
        window_fill: bg_panel,
        window_stroke: Stroke::new(1.0, border_color),
        window_corner_radius: CornerRadius::same(6),
        
        // Widgets
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: bg_panel,
                weak_bg_fill: bg_panel,
                bg_stroke: Stroke::new(1.0, border_color),
                corner_radius: CornerRadius::same(4),
                fg_stroke: Stroke::new(1.0, text_secondary),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: bg_widget,
                weak_bg_fill: bg_widget, 
                bg_stroke: Stroke::new(0.0, Color32::TRANSPARENT), 
                corner_radius: CornerRadius::same(3),
                fg_stroke: Stroke::new(1.0, text_primary),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: bg_widget.lighten(),
                weak_bg_fill: bg_widget.lighten(),
                bg_stroke: Stroke::new(1.0, accent_hover),
                corner_radius: CornerRadius::same(3),
                fg_stroke: Stroke::new(1.0, text_primary),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: accent_active,
                weak_bg_fill: accent_active,
                bg_stroke: Stroke::new(0.0, Color32::TRANSPARENT),
                corner_radius: CornerRadius::same(3),
                fg_stroke: Stroke::new(2.0, Color32::WHITE),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: bg_subpanel,
                weak_bg_fill: bg_subpanel,
                bg_stroke: Stroke::new(1.0, border_color),
                corner_radius: CornerRadius::same(3),
                fg_stroke: Stroke::new(1.0, text_primary),
                expansion: 0.0,
            },
        },

        // Selection
        selection: egui::style::Selection {
            bg_fill: accent_primary.linear_multiply(0.4),
            stroke: Stroke::new(1.0, accent_primary),
        },
        
        // Misc
        hyperlink_color: accent_primary,
        faint_bg_color: bg_subpanel,
        extreme_bg_color: bg_app, 
        code_bg_color: bg_subpanel,
        warn_fg_color: Color32::from_rgb(255, 140, 0),
        error_fg_color: Color32::from_rgb(255, 60, 60),

        ..Default::default()
    };
    
    style.visuals = visuals;

    ctx.set_style(style);
}

/// Helper extension trait for Color32 to make it lighter
pub trait ColorExt {
    fn lighten(&self) -> Self;
}

impl ColorExt for Color32 {
    fn lighten(&self) -> Self {
        let (r, g, b, a) = (self.r(), self.g(), self.b(), self.a());
        Self::from_rgba_premultiplied(
            r.saturating_add(20),
            g.saturating_add(20),
            b.saturating_add(20),
            a
        )
    }
}

/// Color palette for DAW elements
pub struct DawColors;

impl DawColors {
    // Track colors (for automatic assignment)
    pub const TRACK_COLORS: [Color32; 8] = [
        Color32::from_rgb(100, 180, 255),  // Blue
        Color32::from_rgb(255, 150, 100),  // Orange
        Color32::from_rgb(150, 255, 150),  // Green
        Color32::from_rgb(255, 150, 200),  // Pink
        Color32::from_rgb(200, 150, 255),  // Purple
        Color32::from_rgb(255, 255, 150),  // Yellow
        Color32::from_rgb(150, 255, 255),  // Cyan
        Color32::from_rgb(255, 180, 180),  // Salmon
    ];
    
    // Transport colors with adjustable intensity
    pub const PLAY: Color32 = Color32::from_rgb(50, 220, 100);
    pub const STOP: Color32 = Color32::from_rgb(200, 200, 200);
    pub const RECORD: Color32 = Color32::from_rgb(255, 60, 60);
    pub const LOOP: Color32 = Color32::from_rgb(80, 170, 255);
    
    // Meter colors
    pub const METER_GREEN: Color32 = Color32::from_rgb(50, 220, 100);
    pub const METER_YELLOW: Color32 = Color32::from_rgb(255, 200, 50);
    pub const METER_RED: Color32 = Color32::from_rgb(255, 60, 60);
    
    // Grid colors
    pub const GRID_BAR: Color32 = Color32::from_rgb(60, 60, 65);
    pub const GRID_BEAT: Color32 = Color32::from_rgb(40, 40, 45);
    pub const PLAYHEAD: Color32 = Color32::from_rgb(255, 100, 100);
    
    pub fn track_color(index: usize) -> Color32 {
        Self::TRACK_COLORS[index % Self::TRACK_COLORS.len()]
    }
    
    pub fn track_color_u32(index: usize) -> u32 {
        let c = Self::track_color(index);
        ((c.r() as u32) << 24) | ((c.g() as u32) << 16) | ((c.b() as u32) << 8) | 0xFF
    }
}

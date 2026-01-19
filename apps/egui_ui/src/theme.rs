//! Theme Configuration
//!
//! Custom fonts and dark theme for professional DAW look

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Rounding, Style, Visuals};

/// Setup custom fonts for the DAW
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    
    // You can add custom fonts here if needed
    // For now, we'll use the default fonts with adjusted sizes
    
    // Example of how to add a custom font:
    // fonts.font_data.insert(
    //     "inter".to_owned(),
    //     FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")),
    // );
    // fonts.families.get_mut(&FontFamily::Proportional).unwrap()
    //     .insert(0, "inter".to_owned());
    
    ctx.set_fonts(fonts);
}

/// Setup dark theme optimized for DAW workflow
pub fn setup_dark_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    
    // Dark background colors
    let bg_dark = Color32::from_rgb(25, 25, 28);
    let bg_medium = Color32::from_rgb(35, 35, 40);
    let bg_light = Color32::from_rgb(45, 45, 50);
    let bg_widget = Color32::from_rgb(55, 55, 60);
    
    // Accent colors
    let accent = Color32::from_rgb(100, 150, 255);
    let accent_hover = Color32::from_rgb(120, 170, 255);
    let accent_active = Color32::from_rgb(80, 130, 235);
    
    // Text colors
    let text_primary = Color32::from_rgb(230, 230, 230);
    let text_secondary = Color32::from_rgb(160, 160, 160);
    let text_disabled = Color32::from_rgb(100, 100, 100);
    
    // Configure visuals
    let visuals = Visuals {
        dark_mode: true,
        
        // Override colors
        override_text_color: Some(text_primary),
        
        // Widget visuals
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: bg_medium,
                weak_bg_fill: bg_medium,
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 65)),
                rounding: Rounding::same(4.0),
                fg_stroke: egui::Stroke::new(1.0, text_secondary),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: bg_widget,
                weak_bg_fill: bg_widget,
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(70, 70, 75)),
                rounding: Rounding::same(4.0),
                fg_stroke: egui::Stroke::new(1.0, text_primary),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(65, 65, 70),
                weak_bg_fill: Color32::from_rgb(65, 65, 70),
                bg_stroke: egui::Stroke::new(1.0, accent_hover),
                rounding: Rounding::same(4.0),
                fg_stroke: egui::Stroke::new(1.0, text_primary),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: accent_active,
                weak_bg_fill: accent_active,
                bg_stroke: egui::Stroke::new(1.0, accent),
                rounding: Rounding::same(4.0),
                fg_stroke: egui::Stroke::new(2.0, text_primary),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: bg_light,
                weak_bg_fill: bg_light,
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 85)),
                rounding: Rounding::same(4.0),
                fg_stroke: egui::Stroke::new(1.0, text_primary),
                expansion: 0.0,
            },
        },
        
        // Selection
        selection: egui::style::Selection {
            bg_fill: accent.linear_multiply(0.3),
            stroke: egui::Stroke::new(1.0, accent),
        },
        
        // Hyperlinks
        hyperlink_color: accent,
        
        // Backgrounds
        faint_bg_color: bg_dark,
        extreme_bg_color: Color32::from_rgb(15, 15, 18),
        code_bg_color: bg_medium,
        
        // Warnings
        warn_fg_color: Color32::from_rgb(255, 200, 50),
        error_fg_color: Color32::from_rgb(255, 80, 80),
        
        // Window
        window_rounding: Rounding::same(6.0),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 4].into(),
            blur: 8.0,
            spread: 0.0,
            color: Color32::from_black_alpha(80),
        },
        window_fill: bg_medium,
        window_stroke: egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 65)),
        window_highlight_topmost: true,
        
        // Menu
        menu_rounding: Rounding::same(4.0),
        
        // Panel
        panel_fill: bg_dark,
        
        // Popup shadow
        popup_shadow: egui::epaint::Shadow {
            offset: [0, 2].into(),
            blur: 6.0,
            spread: 0.0,
            color: Color32::from_black_alpha(60),
        },
        
        // Resize corner
        resize_corner_size: 12.0,
        
        // Text cursor
        text_cursor: egui::style::TextCursorStyle {
            stroke: egui::Stroke::new(2.0, accent),
            preview: false,
            blink: true,
            on_duration: 0.5,
            off_duration: 0.5,
        },
        
        // Clip rect margin
        clip_rect_margin: 3.0,
        
        // Button frame
        button_frame: true,
        
        // Collapsing header frame
        collapsing_header_frame: true,
        
        // Indent
        indent_has_left_vline: true,
        
        // Striped
        striped: false,
        
        // Slider trailing fill
        slider_trailing_fill: true,
        
        // Handle shape
        handle_shape: egui::style::HandleShape::Circle,
        
        // Interact cursor
        interact_cursor: None,
        
        // Image loading spinners
        image_loading_spinners: true,
        
        // Numeric color space
        numeric_color_space: egui::style::NumericColorSpace::GammaByte,
    };
    
    style.visuals = visuals;
    
    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(8.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.indent = 18.0;
    style.spacing.slider_width = 100.0;
    
    // Animation
    style.animation_time = 0.1;
    
    ctx.set_style(style);
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
    
    // Transport colors
    pub const PLAY: Color32 = Color32::from_rgb(100, 200, 100);
    pub const STOP: Color32 = Color32::from_rgb(200, 200, 200);
    pub const RECORD: Color32 = Color32::from_rgb(255, 80, 80);
    pub const LOOP: Color32 = Color32::from_rgb(100, 180, 255);
    
    // Meter colors
    pub const METER_GREEN: Color32 = Color32::from_rgb(50, 200, 100);
    pub const METER_YELLOW: Color32 = Color32::from_rgb(255, 200, 50);
    pub const METER_RED: Color32 = Color32::from_rgb(255, 50, 50);
    
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

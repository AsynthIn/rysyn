//! Rysyn DAW - egui UI
//!
//! Professional DAW interface built with egui.
//! Communicates with JUCE Audio Core via FFI bridge.

mod app;
mod panels;
mod theme;
mod widgets;

use app::RysynApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1024.0, 600.0])
            .with_title("Rysyn DAW")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Rysyn DAW",
        options,
        Box::new(|cc| {
            // Setup custom fonts and visuals
            theme::setup_custom_fonts(&cc.egui_ctx);
            theme::setup_dark_theme(&cc.egui_ctx);
            Ok(Box::new(RysynApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    // Return default icon for now
    egui::IconData::default()
}

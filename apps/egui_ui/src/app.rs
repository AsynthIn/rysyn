//! Main Application State

use eframe::egui;
use rysyn_ffi_bridge::{init_bridge, get_bridge, Command, StateSnapshot};
use std::sync::mpsc::{channel, Sender, Receiver};

use crate::panels::{
    transport::TransportPanel,
    timeline::TimelinePanel,
    track_list::TrackListPanel,
    mixer::MixerPanel,
    browser::BrowserPanel,
    inspector::InspectorPanel,
};

/// Main application state
pub struct RysynApp {
    /// Current state from audio core
    state: StateSnapshot,
    
    /// Command sender for UI -> Audio Core
    cmd_tx: Sender<Command>,
    cmd_rx: Receiver<Command>,
    
    /// Panel states
    transport: TransportPanel,
    timeline: TimelinePanel,
    track_list: TrackListPanel,
    mixer: MixerPanel,
    browser: BrowserPanel,
    inspector: InspectorPanel,
    
    /// UI state
    show_mixer: bool,
    show_browser: bool,
    show_inspector: bool,
    
    /// Zoom/scroll state
    timeline_zoom: f32,
    timeline_scroll: f64,
}

impl RysynApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize FFI bridge
        init_bridge();
        
        let (cmd_tx, cmd_rx) = channel();
        
        Self {
            state: StateSnapshot::new(),
            cmd_tx,
            cmd_rx,
            transport: TransportPanel::default(),
            timeline: TimelinePanel::default(),
            track_list: TrackListPanel::default(),
            mixer: MixerPanel::default(),
            browser: BrowserPanel::default(),
            inspector: InspectorPanel::default(),
            show_mixer: true,
            show_browser: true,
            show_inspector: false,
            timeline_zoom: 1.0,
            timeline_scroll: 0.0,
        }
    }
    
    fn process_commands(&self) {
        if let Some(bridge) = get_bridge() {
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                let _ = bridge.send_command(cmd);
            }
        }
    }
    
    fn refresh_state(&mut self) {
        if let Some(bridge) = get_bridge() {
            self.state = bridge.get_state();
        }
    }
}

impl eframe::App for RysynApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Refresh state from audio core
        self.refresh_state();
        
        // Process any pending commands
        self.process_commands();
        
        // Request continuous repaint for smooth animation
        ctx.request_repaint();
        
        // Clone cmd_tx for use in closures
        let cmd_tx = self.cmd_tx.clone();
        
        // === Menu Bar ===
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let cmd_tx = cmd_tx.clone();
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        let _ = cmd_tx.send(Command::NewProject);
                        ui.close_menu();
                    }
                    if ui.button("Open Project...").clicked() {
                        // TODO: File dialog
                        ui.close_menu();
                    }
                    if ui.button("Save Project").clicked() {
                        let _ = cmd_tx.send(Command::SaveProject { path: None });
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        let _ = cmd_tx.send(Command::Undo);
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        let _ = cmd_tx.send(Command::Redo);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Track", |ui| {
                    if ui.button("Add Audio Track").clicked() {
                        let _ = cmd_tx.send(Command::CreateTrack { 
                            name: "Audio".to_string(),
                            is_midi: false 
                        });
                        ui.close_menu();
                    }
                    if ui.button("Add MIDI Track").clicked() {
                        let _ = cmd_tx.send(Command::CreateTrack { 
                            name: "MIDI".to_string(),
                            is_midi: true 
                        });
                        ui.close_menu();
                    }
                    if ui.button("Add Instrument Track").clicked() {
                        let _ = cmd_tx.send(Command::CreateTrack { 
                            name: "Instrument".to_string(),
                            is_midi: true 
                        });
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_mixer, "Mixer");
                    ui.checkbox(&mut self.show_browser, "Browser");
                    ui.checkbox(&mut self.show_inspector, "Inspector");
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About Rysyn").clicked() {
                        // TODO: About dialog
                        ui.close_menu();
                    }
                });
                
                // Right-aligned status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("CPU: {:.1}%", self.state.cpu_load * 100.0));
                    ui.separator();
                    ui.label(format!("{} Hz / {} smp", 
                        self.state.sample_rate, 
                        self.state.buffer_size
                    ));
                });
            });
        });
        
        // Clone state for panels (to avoid borrow issues)
        let state = self.state.clone();
        
        // === Transport Bar ===
        let cmd_tx2 = cmd_tx.clone();
        egui::TopBottomPanel::top("transport_bar")
            .min_height(60.0)
            .show(ctx, |ui| {
                self.transport.show(ui, &state, |cmd| { let _ = cmd_tx2.send(cmd); });
            });
        
        // === Bottom Panel (Mixer) ===
        if self.show_mixer {
            let cmd_tx2 = cmd_tx.clone();
            egui::TopBottomPanel::bottom("mixer_panel")
                .resizable(true)
                .default_height(200.0)
                .min_height(100.0)
                .show(ctx, |ui| {
                    self.mixer.show(ui, &state, |cmd| { let _ = cmd_tx2.send(cmd); });
                });
        }
        
        // === Left Panel (Browser) ===
        if self.show_browser {
            let cmd_tx2 = cmd_tx.clone();
            egui::SidePanel::left("browser_panel")
                .resizable(true)
                .default_width(200.0)
                .min_width(150.0)
                .show(ctx, |ui| {
                    self.browser.show(ui, &state, |cmd| { let _ = cmd_tx2.send(cmd); });
                });
        }
        
        // === Right Panel (Inspector) ===
        if self.show_inspector {
            let cmd_tx2 = cmd_tx.clone();
            egui::SidePanel::right("inspector_panel")
                .resizable(true)
                .default_width(250.0)
                .min_width(200.0)
                .show(ctx, |ui| {
                    self.inspector.show(ui, &state, |cmd| { let _ = cmd_tx2.send(cmd); });
                });
        }
        
        // === Central Area (Track List + Timeline) ===
        let cmd_tx2 = cmd_tx.clone();
        let cmd_tx3 = cmd_tx.clone();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Track list (left side, fixed width)
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .show(ui, |ui| {
                        ui.set_width(200.0);
                        self.track_list.show(
                            ui, 
                            &state, 
                            |cmd| { let _ = cmd_tx2.send(cmd); }
                        );
                    });
                
                // Timeline (right side, fills remaining space)
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(false), |ui| {
                    self.timeline.show(
                        ui, 
                        &state, 
                        |cmd| { let _ = cmd_tx3.send(cmd); },
                        &mut self.timeline_zoom,
                        &mut self.timeline_scroll
                    );
                });
            });
        });
    }
}

//! Browser Panel
//!
//! File browser for audio files and VST plugins

use eframe::egui::{self, Color32, RichText, Sense, Vec2};
use rysyn_ffi_bridge::{Command, StateSnapshot};
use std::path::PathBuf;

#[derive(Default, PartialEq)]
pub enum BrowserTab {
    #[default]
    Files,
    Plugins,
    Presets,
}

#[derive(Default)]
pub struct BrowserPanel {
    current_tab: BrowserTab,
    current_path: PathBuf,
    search_query: String,
    selected_item: Option<String>,
}

impl BrowserPanel {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        mut send_cmd: impl FnMut(Command),
    ) {
        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tab, BrowserTab::Files, "Files");
            ui.selectable_value(&mut self.current_tab, BrowserTab::Plugins, "Plugins");
            ui.selectable_value(&mut self.current_tab, BrowserTab::Presets, "Presets");
        });
        
        ui.separator();
        
        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search...")
                    .desired_width(ui.available_width())
            );
        });
        
        ui.add_space(4.0);
        
        match self.current_tab {
            BrowserTab::Files => self.show_files_tab(ui, state, &mut send_cmd),
            BrowserTab::Plugins => self.show_plugins_tab(ui, state, &mut send_cmd),
            BrowserTab::Presets => self.show_presets_tab(ui, state, &mut send_cmd),
        }
    }
    
    fn show_files_tab(
        &mut self,
        ui: &mut egui::Ui,
        _state: &StateSnapshot,
        send_cmd: &mut impl FnMut(Command),
    ) {
        // Path breadcrumb
        ui.horizontal(|ui| {
            if ui.small_button("🏠").clicked() {
                if let Some(home) = dirs::home_dir() {
                    self.current_path = home;
                }
            }
            
            if ui.small_button("⬆").clicked() {
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                }
            }
            
            ui.label(
                RichText::new(self.current_path.display().to_string())
                    .size(10.0)
                    .color(Color32::GRAY)
            );
        });
        
        ui.separator();
        
        // File list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Ok(entries) = std::fs::read_dir(&self.current_path) {
                    let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                    items.sort_by(|a, b| {
                        let a_dir = a.path().is_dir();
                        let b_dir = b.path().is_dir();
                        match (a_dir, b_dir) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.file_name().cmp(&b.file_name()),
                        }
                    });
                    
                    for entry in items {
                        let path = entry.path();
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        
                        // Filter by search
                        if !self.search_query.is_empty() 
                            && !name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                            continue;
                        }
                        
                        let is_dir = path.is_dir();
                        let is_audio = is_audio_file(&path);
                        
                        // Skip non-audio files (but show directories)
                        if !is_dir && !is_audio && !self.search_query.is_empty() {
                            continue;
                        }
                        
                        let icon = if is_dir {
                            "📁"
                        } else if is_audio {
                            "🎵"
                        } else {
                            "📄"
                        };
                        
                        let is_selected = self.selected_item.as_ref() == Some(&name);
                        
                        let response = ui.add(
                            egui::SelectableLabel::new(
                                is_selected,
                                format!("{} {}", icon, name)
                            )
                        );
                        
                        if response.clicked() {
                            self.selected_item = Some(name.clone());
                        }
                        
                        if response.double_clicked() {
                            if is_dir {
                                self.current_path = path.clone();
                                self.selected_item = None;
                            } else if is_audio {
                                // Import audio file
                                send_cmd(Command::ImportAudioFile { 
                                    path: path.to_string_lossy().to_string()
                                });
                            }
                        }
                        
                        // Drag to timeline
                        if is_audio {
                            response.on_hover_text("Drag to timeline or double-click to import");
                        }
                    }
                } else {
                    ui.label(
                        RichText::new("Cannot read directory")
                            .color(Color32::from_rgb(255, 100, 100))
                    );
                }
            });
    }
    
    fn show_plugins_tab(
        &mut self,
        ui: &mut egui::Ui,
        state: &StateSnapshot,
        send_cmd: &mut impl FnMut(Command),
    ) {
        // Plugin categories
        ui.horizontal(|ui| {
            ui.label("Category:");
            ui.selectable_label(true, "All");
            ui.selectable_label(false, "Instruments");
            ui.selectable_label(false, "Effects");
        });
        
        ui.separator();
        
        // Rescan button
        ui.horizontal(|ui| {
            if ui.button("🔄 Rescan Plugins").clicked() {
                send_cmd(Command::RescanPlugins);
            }
        });
        
        ui.add_space(4.0);
        
        // Plugin list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for plugin in &state.available_plugins {
                    // Filter by search
                    if !self.search_query.is_empty() 
                        && !plugin.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                        continue;
                    }
                    
                    let icon = if plugin.is_instrument { "🎹" } else { "🎛" };
                    let format_text = match plugin.format.as_str() {
                        "VST3" => "VST3",
                        "AU" => "AU",
                        "CLAP" => "CLAP",
                        _ => "?",
                    };
                    
                    let is_selected = self.selected_item.as_ref() == Some(&plugin.id);
                    
                    let response = ui.add(
                        egui::SelectableLabel::new(
                            is_selected,
                            format!("{} {} [{}]", icon, &plugin.name, format_text)
                        )
                    );
                    
                    if response.clicked() {
                        self.selected_item = Some(plugin.id.clone());
                    }
                    
                    if response.double_clicked() {
                        // Load plugin on selected track
                        send_cmd(Command::LoadPlugin { 
                            track_id: state.tracks.first().map(|t| t.id).unwrap_or(0),
                            plugin_id: plugin.id.clone(),
                            slot: None,
                        });
                    }
                    
                    response.on_hover_ui(|ui| {
                        ui.label(&plugin.name);
                        ui.label(format!("Manufacturer: {}", &plugin.manufacturer));
                        ui.label(format!("Format: {}", format_text));
                        ui.label(format!("Path: {}", &plugin.path));
                    });
                }
                
                if state.available_plugins.is_empty() {
                    ui.label(
                        RichText::new("No plugins found. Click 'Rescan Plugins'.")
                            .color(Color32::GRAY)
                    );
                }
            });
    }
    
    fn show_presets_tab(
        &mut self,
        ui: &mut egui::Ui,
        _state: &StateSnapshot,
        _send_cmd: &mut impl FnMut(Command),
    ) {
        ui.label(
            RichText::new("Presets browser coming soon...")
                .color(Color32::GRAY)
        );
        
        // Placeholder for preset browser
        // Would show:
        // - Factory presets
        // - User presets
        // - Preset categories
        // - Favorites
    }
}

fn is_audio_file(path: &std::path::Path) -> bool {
    let extensions = ["wav", "mp3", "flac", "aiff", "aif", "ogg", "m4a"];
    
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

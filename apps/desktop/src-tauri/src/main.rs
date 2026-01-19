#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use rysyn_audio_engine::AudioEngine;
use rysyn_project::{Track, AudioItem};

struct AppState {
    audio_engine: Mutex<AudioEngine>,
}

#[tauri::command]
fn start_engine(state: tauri::State<AppState>) -> String {
    let mut engine = state.audio_engine.lock().unwrap();
    match engine.start() {
        Ok(_) => "Audio Engine Started".to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

#[tauri::command]
fn stop_engine(state: tauri::State<AppState>) -> String {
    let mut engine = state.audio_engine.lock().unwrap();
    engine.stop();
    "Audio Engine Stopped".to_string()
}

#[tauri::command]
fn play_transport(state: tauri::State<AppState>) -> Result<(), String> {
    let engine = state.audio_engine.lock().unwrap();
    let project = engine.get_project();
    let mut p = project.lock().unwrap();
    p.is_playing = true;
    Ok(())
}

#[tauri::command]
fn pause_transport(state: tauri::State<AppState>) -> Result<(), String> {
    let engine = state.audio_engine.lock().unwrap();
    let project = engine.get_project();
    let mut p = project.lock().unwrap();
    p.is_playing = false;
    Ok(())
}

#[tauri::command]
fn add_track(state: tauri::State<AppState>) -> Result<String, String> {
    let engine = state.audio_engine.lock().unwrap();
    let project_mutex = engine.get_project();
    let mut project = project_mutex.lock().unwrap();
    
    let next_id = (project.tracks.len() as u32) + 1;
    let new_track = Track::new(next_id, format!("Track {}", next_id));
    project.tracks.push(new_track);
    
    Ok(format!("Added Track {}", next_id))
}

#[tauri::command]
fn import_audio(state: tauri::State<AppState>, track_id: u32, path: String) -> Result<String, String> {
    let engine = state.audio_engine.lock().unwrap();
    
    // Load into engine cache
    engine.load_audio_file(&path)?;

    // Add Item to Project
    let project_mutex = engine.get_project();
    let mut project = project_mutex.lock().unwrap();
    
    if let Some(track) = project.tracks.iter_mut().find(|t| t.id == track_id) {
        let item = AudioItem {
            id: (track.items.len() as u32) + 1,
            start_time: 0.0,
            duration: 10.0, // Hardcoded for now, should read from file duration
            source_path: path.clone(),
            start_offset: 0.0,
        };
        track.items.push(item);
        Ok(format!("Imported {} to Track {}", path, track_id))
    } else {
        Err("Track not found".to_string())
    }
}

#[tauri::command]
fn get_tracks(state: tauri::State<AppState>) -> Result<Vec<Track>, String> {
    let engine = state.audio_engine.lock().unwrap();
    let project_mutex = engine.get_project();
    let project = project_mutex.lock().unwrap();
    
    Ok(project.tracks.clone())
}

#[tauri::command]
fn get_transport_pos(state: tauri::State<AppState>) -> Result<f64, String> {
    let engine = state.audio_engine.lock().unwrap();
    let project_mutex = engine.get_project();
    let project = project_mutex.lock().unwrap();
    Ok(project.playhead_pos)
}


fn main() {
    tauri::Builder::default()
        .manage(AppState {
            audio_engine: Mutex::new(AudioEngine::new()),
        })
        .invoke_handler(tauri::generate_handler![
            start_engine, 
            stop_engine, 
            play_transport, 
            pause_transport,
            add_track, 
            import_audio,
            get_tracks,
            get_transport_pos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! Python Scripting Integration
//! 
//! Provides PyO3 bindings for Rysyn DAW scripting.
//! Users can control the DAW, automate tasks, and create custom workflows.

use pyo3::prelude::*;
use rysyn_ffi_bridge::{get_bridge, init_bridge, Command};

/// Python module: rysyn
/// 
/// Example usage:
/// ```python
/// import rysyn
/// 
/// # Transport control
/// rysyn.play()
/// rysyn.stop()
/// rysyn.set_bpm(140.0)
/// 
/// # Get state
/// state = rysyn.get_state()
/// print(f"Playing: {state['transport']['is_playing']}")
/// 
/// # Create track
/// rysyn.create_track("Synth Lead", is_midi=True)
/// ```
#[pymodule]
fn rysyn(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize bridge if not already done
    init_bridge();
    
    // Transport
    m.add_function(wrap_pyfunction!(play, m)?)?;
    m.add_function(wrap_pyfunction!(pause, m)?)?;
    m.add_function(wrap_pyfunction!(stop, m)?)?;
    m.add_function(wrap_pyfunction!(toggle_record, m)?)?;
    m.add_function(wrap_pyfunction!(toggle_loop, m)?)?;
    m.add_function(wrap_pyfunction!(set_bpm, m)?)?;
    m.add_function(wrap_pyfunction!(set_playhead, m)?)?;
    m.add_function(wrap_pyfunction!(set_loop_region, m)?)?;
    
    // Tracks
    m.add_function(wrap_pyfunction!(create_track, m)?)?;
    m.add_function(wrap_pyfunction!(delete_track, m)?)?;
    m.add_function(wrap_pyfunction!(set_track_volume, m)?)?;
    m.add_function(wrap_pyfunction!(set_track_pan, m)?)?;
    m.add_function(wrap_pyfunction!(set_track_mute, m)?)?;
    m.add_function(wrap_pyfunction!(set_track_solo, m)?)?;
    
    // Plugins
    m.add_function(wrap_pyfunction!(rescan_plugins, m)?)?;
    m.add_function(wrap_pyfunction!(load_plugin, m)?)?;
    m.add_function(wrap_pyfunction!(remove_plugin, m)?)?;
    
    // State
    m.add_function(wrap_pyfunction!(get_state, m)?)?;
    m.add_function(wrap_pyfunction!(get_transport, m)?)?;
    m.add_function(wrap_pyfunction!(get_tracks, m)?)?;
    m.add_function(wrap_pyfunction!(get_plugins, m)?)?;
    
    // Project
    m.add_function(wrap_pyfunction!(new_project, m)?)?;
    m.add_function(wrap_pyfunction!(save_project, m)?)?;
    m.add_function(wrap_pyfunction!(load_project, m)?)?;
    
    Ok(())
}

// === Transport Functions ===

#[pyfunction]
fn play() -> PyResult<bool> {
    send_command(Command::Play)
}

#[pyfunction]
fn pause() -> PyResult<bool> {
    send_command(Command::Pause)
}

#[pyfunction]
fn stop() -> PyResult<bool> {
    send_command(Command::Stop)
}

#[pyfunction]
fn toggle_record() -> PyResult<bool> {
    send_command(Command::ToggleRecord)
}

#[pyfunction]
fn toggle_loop() -> PyResult<bool> {
    send_command(Command::ToggleLoop)
}

#[pyfunction]
fn set_bpm(bpm: f64) -> PyResult<bool> {
    send_command(Command::SetTempo { bpm })
}

#[pyfunction]
fn set_playhead(beats: f64) -> PyResult<bool> {
    send_command(Command::SetPlayhead { beats })
}

#[pyfunction]
fn set_loop_region(start_beats: f64, end_beats: f64) -> PyResult<bool> {
    send_command(Command::SetLoopRegion { start_beats, end_beats })
}

// === Track Functions ===

#[pyfunction]
#[pyo3(signature = (name, is_midi=false))]
fn create_track(name: String, is_midi: bool) -> PyResult<bool> {
    send_command(Command::CreateTrack { name, is_midi })
}

#[pyfunction]
fn delete_track(track_id: u32) -> PyResult<bool> {
    send_command(Command::DeleteTrack { track_id })
}

#[pyfunction]
fn set_track_volume(track_id: u32, volume: f32) -> PyResult<bool> {
    send_command(Command::SetTrackVolume { track_id, volume })
}

#[pyfunction]
fn set_track_pan(track_id: u32, pan: f32) -> PyResult<bool> {
    send_command(Command::SetTrackPan { track_id, pan })
}

#[pyfunction]
fn set_track_mute(track_id: u32, muted: bool) -> PyResult<bool> {
    send_command(Command::SetTrackMute { track_id, muted })
}

#[pyfunction]
fn set_track_solo(track_id: u32, soloed: bool) -> PyResult<bool> {
    send_command(Command::SetTrackSolo { track_id, soloed })
}

// === Plugin Functions ===

#[pyfunction]
fn rescan_plugins() -> PyResult<bool> {
    send_command(Command::RescanPlugins)
}

#[pyfunction]
#[pyo3(signature = (track_id, plugin_id, slot=None))]
fn load_plugin(track_id: u32, plugin_id: String, slot: Option<u32>) -> PyResult<bool> {
    send_command(Command::LoadPlugin { track_id, plugin_id, slot })
}

#[pyfunction]
fn remove_plugin(track_id: u32, slot: u32) -> PyResult<bool> {
    send_command(Command::RemovePlugin { track_id, slot })
}

// === State Functions ===

#[pyfunction]
fn get_state(py: Python<'_>) -> PyResult<PyObject> {
    let bridge = get_bridge().ok_or_else(|| {
        pyo3::exceptions::PyRuntimeError::new_err("Bridge not initialized")
    })?;
    
    let state = bridge.get_state();
    let json = serde_json::to_string(&state)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    
    // Parse JSON to Python dict
    let json_module = py.import("json")?;
    let dict = json_module.call_method1("loads", (json,))?;
    Ok(dict.into_pyobject(py)?.unbind().into())
}

#[pyfunction]
fn get_transport(py: Python<'_>) -> PyResult<PyObject> {
    let state = get_state(py)?;
    let state_bound = state.bind(py);
    let dict = state_bound.get_item("transport")?;
    Ok(dict.into_pyobject(py)?.unbind().into())
}

#[pyfunction]
fn get_tracks(py: Python<'_>) -> PyResult<PyObject> {
    let state = get_state(py)?;
    let state_bound = state.bind(py);
    let tracks = state_bound.get_item("tracks")?;
    Ok(tracks.into_pyobject(py)?.unbind().into())
}

#[pyfunction]
fn get_plugins(py: Python<'_>) -> PyResult<PyObject> {
    let state = get_state(py)?;
    let state_bound = state.bind(py);
    let plugins = state_bound.get_item("available_plugins")?;
    Ok(plugins.into_pyobject(py)?.unbind().into())
}

// === Project Functions ===

#[pyfunction]
fn new_project() -> PyResult<bool> {
    send_command(Command::NewProject)
}

#[pyfunction]
#[pyo3(signature = (path=None))]
fn save_project(path: Option<String>) -> PyResult<bool> {
    send_command(Command::SaveProject { path })
}

#[pyfunction]
fn load_project(path: String) -> PyResult<bool> {
    send_command(Command::LoadProject { path })
}

// === Helper Functions ===

fn send_command(cmd: Command) -> PyResult<bool> {
    let bridge = get_bridge().ok_or_else(|| {
        pyo3::exceptions::PyRuntimeError::new_err("Bridge not initialized")
    })?;
    
    bridge.send_command(cmd)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;
    Ok(true)
}

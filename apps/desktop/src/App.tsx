import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Track {
  id: number;
  name: string;
  volume: number;
  pan: number;
  mute: boolean;
  solo: boolean;
}

function App() {
  const [status, setStatus] = useState("Ready");
  const [tracks, setTracks] = useState<Track[]>([]);

  async function toggleAudio() {
    try {
      const msg = await invoke("toggle_audio");
      setStatus(msg as string);
    } catch (e) {
      setStatus("Error: " + e);
    }
  }

  async function addTrack() {
    try {
      await invoke("add_track");
      refreshTracks();
    } catch (e) {
      console.error(e);
    }
  }

  async function refreshTracks() {
    try {
      const t = await invoke("get_tracks");
      setTracks(t as Track[]);
    } catch (e) {
      console.error(e);
    }
  }

  useEffect(() => {
    refreshTracks();
  }, []);

  return (
    <div className="container" style={{ padding: 20, fontFamily: 'sans-serif', background: '#222', color: '#eee', minHeight: '100vh' }}>
      <h1>Rysyn DAW</h1>
      <div style={{ marginBottom: 20 }}>
        <button onClick={toggleAudio} style={{ padding: '8px 16px', marginRight: 10 }}>Start Audio Engine</button>
        <button onClick={addTrack} style={{ padding: '8px 16px' }}>+ Add Track</button>
      </div>
      <p>Status: {status}</p>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        {tracks.map(t => (
          <div key={t.id} style={{ background: '#333', padding: 10, borderRadius: 4, display: 'flex', alignItems: 'center' }}>
            <div style={{ width: 30, textAlign: 'center', background: '#444', marginRight: 10 }}>{t.id}</div>
            <div style={{ flex: 1, fontWeight: 'bold' }}>{t.name}</div>
            <div style={{ width: 100 }}>Vol: {t.volume.toFixed(1)}</div>
            <div style={{ width: 100 }}>Pan: {t.pan.toFixed(1)}</div>
            <div style={{ width: 50, color: t.mute ? 'red' : '#666' }}>M</div>
            <div style={{ width: 50, color: t.solo ? 'yellow' : '#666' }}>S</div>
          </div>
        ))}
        {tracks.length === 0 && <div style={{ color: '#666', fontStyle: 'italic' }}>No tracks yet. Add one to hear sound.</div>}
      </div>
    </div>
  );
}

export default App;

import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Track {
  id: number;
  name: string;
  volume: number;
  pan: number;
  mute: boolean;
  solo: boolean;
  items: any[];
  pattern_instances: any[];
  instrument: any;
}

function App() {
  const [status, setStatus] = useState("Ready");
  const [tracks, setTracks] = useState<Track[]>([]);
  const [playhead, setPlayhead] = useState(0.0);
  const [isPlaying, setIsPlaying] = useState(false);
  const reqRef = useRef<number>();

  async function startEngine() {
    try {
      const msg = await invoke("start_engine");
      setStatus(msg as string);
    } catch (e) { setStatus("Error: " + e); }
  }

  async function stopEngine() {
    try {
      await invoke("stop_engine");
      setStatus("Engine Stopped");
    } catch (e) { setStatus("Error: " + e); }
  }

  async function play() {
    await invoke("play_transport");
    setIsPlaying(true);
  }

  async function pause() {
    await invoke("pause_transport");
    setIsPlaying(false);
  }

  async function addTrack() {
    await invoke("add_track");
    refreshTracks();
  }

  async function importAudio(trackId: number) {
    const path = prompt("Enter absolute path to audio file (wav/mp3/flac):");
    if (path) {
      try {
        const msg = await invoke("import_audio", { trackId, path });
        setStatus(msg as string);
        refreshTracks();
      } catch (e) {
        setStatus("Import Error: " + e);
      }
    }
  }

  async function refreshTracks() {
    const t = await invoke("get_tracks");
    setTracks(t as Track[]);
  }

  async function updatePlayhead() {
    if (isPlaying) {
      try {
        const pos = await invoke("get_transport_pos");
        setPlayhead(pos as number);
      } catch (e) {}
    }
  }

  async function createDemoPattern() {
    try {
      // 1. Create Pattern Arp
      const patId = (await invoke("create_pattern", { name: "Arp", length: 4.0 })) as number;
      // 2. Add Notes
      await invoke("add_note", { patternId: patId, start: 0.0, duration: 1.0, key: 60, val: 100 });
      await invoke("add_note", { patternId: patId, start: 1.0, duration: 1.0, key: 64, val: 100 });
      await invoke("add_note", { patternId: patId, start: 2.0, duration: 1.0, key: 67, val: 100 });
      await invoke("add_note", { patternId: patId, start: 3.0, duration: 1.0, key: 72, val: 100 });
      // 3. Add Track
      await invoke("add_track");
      const currentTracks = (await invoke("get_tracks")) as Track[];
      const trackId = currentTracks[currentTracks.length - 1].id;
      // 4. Set Instrument
      await invoke("set_track_instrument", { trackId, instType: "SimpleSine" });
      // 5. Place Pattern
      await invoke("place_pattern", { trackId, patternId: patId, startTime: 0.0 });
      
      setStatus("Demo Pattern Created on Track " + trackId);
      refreshTracks();
    } catch (e) {
      setStatus("Error creating demo: " + e);
    }
  }

  useEffect(() => {
    refreshTracks();
    const interval = setInterval(updatePlayhead, 100);
    return () => clearInterval(interval);
  }, [isPlaying]);

  return (
    <div className="container" style={{ padding: 20, fontFamily: 'sans-serif', background: '#222', color: '#eee', minHeight: '100vh' }}>
      <h1>Rysyn DAW</h1>
      
      <div style={{ marginBottom: 20, padding: 10, background: '#333', borderRadius: 8 }}>
        <h3>Transport</h3>
        <div style={{ display: 'flex', gap: 10, marginBottom: 10 }}>
          <button onClick={startEngine}>BoOt Engine</button>
          <button onClick={stopEngine}>KiLl Engine</button>
          <div style={{ width: 20 }} />
          <button onClick={play} style={{ background: isPlaying ? 'green' : '' }}>Play</button>
          <button onClick={pause}>Pause</button>
        </div>
        <div style={{ fontSize: 24, fontFamily: 'monospace' }}>
          {playhead.toFixed(3)} s
        </div>
      </div>

      <div style={{ marginBottom: 10 }}>
        <button onClick={addTrack}>+ Add Track</button>
        <button onClick={createDemoPattern} style={{ marginLeft: 10 }}>+ Add Pattern Demo</button>
      </div>

      <p>Status: {status}</p>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        {tracks.map(t => (
          <div key={t.id} style={{ background: '#333', padding: 10, borderRadius: 4, display: 'flex', alignItems: 'center' }}>
            <div style={{ width: 30, textAlign: 'center', background: '#444', marginRight: 10 }}>{t.id}</div>
            <div style={{ width: 100, fontWeight: 'bold' }}>{t.name}</div>
            
            <button onClick={() => importAudio(t.id)} style={{ fontSize: 12, marginRight: 20 }}>Import Audio</button>

            <div style={{ flex: 1, display: 'flex', gap: 2, height: 40, background: '#111', position: 'relative', overflow: 'hidden' }}>
              {/* Pattern Visualization */}
              {t.pattern_instances && t.pattern_instances.map((pi: any, i: number) => (
                <div key={'p'+i} style={{ 
                  position: 'absolute', 
                  left: pi.start_time * 20, 
                  width: 80, // Arbitrary width for now
                  height: '100%', 
                  background: 'orange',
                  border: '1px solid #fff',
                  opacity: 0.8
                }}>
                  <span style={{ fontSize: 10, color: 'black', padding: 2 }}>
                    Pat {pi.pattern_id}
                  </span>
                </div>
              ))}
              {/* Simple Timeline Visualization for Track Items */}
              {t.items && t.items.map((item: any, i: number) => (
                <div key={i} style={{ 
                  position: 'absolute', 
                  left: item.start_time * 20, // 20px per second zoom
                  width: item.duration * 20, 
                  height: '100%', 
                  background: '#4a90e2',
                  border: '1px solid #fff',
                  opacity: 0.8
                }} title={item.source_path}>
                  <span style={{ fontSize: 10, color: 'white', padding: 2, display: 'block', overflow: 'hidden', whiteSpace: 'nowrap' }}>
                    {item.source_path.split('/').pop()}
                  </span>
                </div>
              ))}
              {/* Playhead Cursor */}
              <div style={{
                position: 'absolute',
                left: playhead * 20,
                top: 0,
                bottom: 0,
                width: 2,
                background: 'red',
                zIndex: 10
              }} />
            </div>

            <div style={{ width: 50, marginLeft: 10 }}>Vol: {t.volume}</div>
          </div>
        ))}
        {tracks.length === 0 && <div style={{ color: '#666', fontStyle: 'italic' }}>No tracks yet.</div>}
      </div>
    </div>
  );
}

export default App;

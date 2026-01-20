Rysyn - Professional DAW
========================

A **next-generation Digital Audio Workstation** combining:

- **JUCE 8.x** (C++) for rock-solid audio engine & plugin hosting
- **egui 0.31** (Rust) for immediate-mode reactive UI
- **Rust FFI Bridge** for seamless realtime-safe interop
- **Python scripting** (PyO3) for automation & extensibility

Quick Start
===========

**Requirements:**

- Rust 1.70+
- CMake 3.22+
- JUCE 8.0.4 (auto-downloaded via CMake FetchContent)
- libflac, libvorbis, ALSA (Linux)

**Build:**

.. code-block:: bash

    # Build Rust FFI bridge (creates static lib)
    cargo build --release -p rysyn_ffi_bridge

    # Build JUCE audio core
    cd apps/juce_core
    mkdir build && cd build
    cmake ..
    cmake --build . -j$(nproc)

    # Run egui UI
    cd ../.. && cargo run --release --bin rysyn

**Test audio core:**

.. code-block:: bash

    # Standalone JUCE app - initializes audio, scans plugins, tests transport
    ./apps/juce_core/build/rysyn_juce_core_artefacts/rysyn_juce_core

Data Flow Principles
====================

**Golden Rule: UI never touches realtime threads**

1. **Per-Frame Snapshot (UI Read-Only)**
   
   - Every frame, UI calls ``rysyn_get_state_json()``
   - Receives JSON snapshot of entire DAW state:
   
     .. code-block:: json
   
         {
           "transport": {
             "is_playing": true,
             "playhead_beats": 24.5,
             "bpm": 120.0,
             "tempo": 120.0
           },
           "tracks": [
             {
               "id": 1,
               "name": "Kick",
               "volume": 1.0,
               "pan": 0.0,
               "is_muted": false,
               "is_selected": false,
               "plugins": ["AIDA-X", null, null, null, ...]
             }
           ],
           "meters": {
             "master_l": 0.45,
             "master_r": 0.42,
             "track_levels_l": [0.32, 0.18, 0.0, ...]
           },
           "available_plugins": [
             {"id": "Cardinal", "name": "Cardinal", "format": "VST3", ...},
             {"id": "SurgeXT", "name": "Surge XT", ...}
           ]
         }
   
   - UI renders based on this snapshot (no state mutations)
   - Metric updates (VU meters, playhead) appear smoothly

2. **Command Queue (UI → Audio)**
   
   - User clicks "Play" → UI sends JSON command:
   
     .. code-block:: json
   
         {"Play": null}
   
   - User changes BPM → UI sends:
   
     .. code-block:: json
   
         {"SetTempo": {"bpm": 140.0}}
   
   - Commands are serialized via serde, sent through ``rysyn_send_command_json()``
   - Queued in crossbeam channel (lockfree MPMC)

3. **JUCE Message Thread (Command Processing)**
   
   - JUCE timer fires every ~16ms (on message thread, NOT audio thread)
   - Calls ``rysyn_recv_command_json()`` repeatedly
   - Parses each command JSON and dispatches:
   
     - ``Play`` → ``audioCore->play()``
     - ``SetTempo`` → ``transport->setBpm()``
     - ``LoadPlugin`` → ``track->loadPlugin(slotId, pluginId)``
     - etc.
   
   - After processing, calls ``rysyn_update_state_json()`` with fresh state

4. **Realtime Audio Thread (Read-Only)**
   
   - Audio callback: ``AudioSource::getNextAudioBlock()``
   - Reads ONLY:
     - Atomic<bool> playing
     - Atomic<double> bpm, playheadBeats
     - Mutable track mix controls (already atomic)
   - Produces audio + updates peak meters (atomics)
   - **ZERO allocations, ZERO locks**

Implementation Status
=====================

✅ **Completed**

- Audio device I/O (ALSA on Linux, CoreAudio on Mac, WinMM on Windows via JUCE)
- VST3 plugin scanning & hosting (84 plugins found: Cardinal, Surge XT, Dexed, etc.)
- Transport (play/pause/stop, BPM, loop region, playhead seek)
- Track management (create/delete, volume/pan/mute/solo)
- egui UI layout (Transport, Timeline, TrackList, Mixer, Browser, Inspector panels)
- FFI bridge (C exports + Rust internals)
- JSON state serialization (serde_json)
- Command queue (crossbeam-channel)
- Python scripting skeleton (PyO3 bindings ready)

⏳ **TODO (Post-MVP)**

- Audio clip playback + MIDI sequencing
- Recording (mic input → clips)
- Offline rendering
- Project save/load (.rysyn format)
- MIDI I/O + controller mappings
- Audio editing (time-stretching, pitch, etc.)
- Undo/Redo system
- Preferences + audio device selection UI
- Keyboard shortcuts
- Plugin presets
- Track/clip coloring + naming
- Meter/analyzer tools
- Built-in effects (EQ, Compressor, Reverb)

Testing
=======

**Audio Core Standalone Test**

.. code-block:: bash

    ./apps/juce_core/build/rysyn_juce_core_artefacts/rysyn_juce_core

Output::

    Audio Core initialized successfully!
    Sample Rate: 44100 Hz
    Buffer Size: 512 samples
    Added track with ID: 1
    Scanning for VST3 plugins...
    Found 84 plugins:
      - AIDA-X (AIDA DSP)
      - Cardinal (DISTRHO)
      - Dexed (Digital Suburban)
      - Surge XT (Surge Synth Team)
      ... and 80 more
    Starting playback test (BPM: 120)...
    Playback stopped.

**UI Build**

.. code-block:: bash

    cargo build --release --bin rysyn
    # Runs immediately with all UI panels visible
    # Transport responsive: play/pause buttons work
    # Track list: can see default track
    # Mixer: volume/pan controls present
    # Browser: plugin list loads from JUCE state

**Python Scripting** (Post-MVP)

.. code-block:: python

    import rysyn
    
    # Transport
    rysyn.play()
    rysyn.set_bpm(140.0)
    rysyn.set_loop_region(0.0, 8.0)
    rysyn.stop()
    
    # Tracks
    rysyn.create_track("Bass", is_midi=True)
    rysyn.set_track_volume(track_id=1, volume=0.8)
    
    # Plugins
    plugins = rysyn.get_plugins()
    rysyn.load_plugin(track_id=1, plugin_id="Surge XT", slot=0)
    
    # State inspection
    state = rysyn.get_state()
    print(state["transport"]["bpm"])


Key Architectural Decisions
============================

1. **Why JUCE?**
   - Battle-tested audio I/O across Windows/Mac/Linux
   - VST3 hosting built-in (same code works everywhere)
   - Lightweight compared to full framework overhead
   - Used by REAPER, Bitwig, Ableton internals

2. **Why egui (Rust)?**
   - Immediate-mode = easy to sync with async state
   - Blazing fast iteration (no borrow checker headaches on UI)
   - Integrates seamlessly with Rust ecosystem
   - Alternative: C++ UI in Qt, but then lose Rust benefits

3. **Why JSON for state/commands?**
   - Human-readable debugging
   - Extensible (add fields without breaking ABI)
   - Easy to script/log/replay
   - Same approach as REAPER's action system

4. **Why FFI instead of embedding?**
   - Clean boundary: audio core is C++, UI is Rust
   - Either can be replaced/upgraded independently
   - Plugin presets, configs can be pure JSON
   - Easier to test each layer separately

5. **Why not lock-free everywhere?**
   - Message thread already has 16ms latency budget (safe for commands)
   - Audio thread only reads atomics (zero overhead)
   - Simplifies implementation vs. wait-free queues

Roadmap
=======

**Phase 1 (MVP - Current)**

- [done] Audio core + VST3 hosting
- [done] Basic UI framework
- [done] Command/state system
- [WIP] Audio playback (in progress)

**Phase 2 (Beta)**

- Recording
- MIDI sequencing
- Clip editing
- Project save/load

**Phase 3 (v1.0)**

- Complex time-stretching
- Built-in effects suite
- Advanced plugin routing
- Undo/Redo
- User presets

Contributing
============

See `CONTRIBUTING.md` (coming soon)

License
=======

MIT License - See `LICENSE` file

Inspired by REAPER's architecture and Bitwig's cross-platform approach.
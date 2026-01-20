#include "AudioCore.h"
#include "TrackProcessor.h"
#include "PluginHost.h"

namespace rysyn {

AudioCore::AudioCore()
{
    deviceManager = std::make_unique<juce::AudioDeviceManager>();
    sourcePlayer = std::make_unique<juce::AudioSourcePlayer>();
    pluginHost = std::make_unique<PluginHost>();
}

AudioCore::~AudioCore()
{
    shutdown();
}

bool AudioCore::initialize()
{
#ifdef RYSYN_FFI_AVAILABLE
    rysyn_init();
#endif

    // Initialize audio device with default settings
    auto result = deviceManager->initialiseWithDefaultDevices(2, 2);
    if (result.isNotEmpty()) {
        DBG("Audio device init failed: " << result);
        return false;
    }

    // Register this as the audio source
    sourcePlayer->setSource(this);
    deviceManager->addAudioCallback(sourcePlayer.get());
    deviceManager->addChangeListener(this);

    // Start timer for FFI command processing (60 Hz)
    startTimer(16);

    DBG("AudioCore initialized successfully");
    return true;
}

void AudioCore::shutdown()
{
    stopTimer();
    stop();

    deviceManager->removeChangeListener(this);
    deviceManager->removeAudioCallback(sourcePlayer.get());
    sourcePlayer->setSource(nullptr);

    {
        juce::ScopedLock lock(trackLock);
        tracks.clear();
    }

    DBG("AudioCore shutdown");
}

juce::StringArray AudioCore::getAvailableDevices() const
{
    juce::StringArray devices;
    auto* currentType = deviceManager->getCurrentDeviceTypeObject();
    if (currentType) {
        devices = currentType->getDeviceNames();
    }
    return devices;
}

bool AudioCore::setAudioDevice(const juce::String& deviceName)
{
    auto setup = deviceManager->getAudioDeviceSetup();
    setup.outputDeviceName = deviceName;
    auto result = deviceManager->setAudioDeviceSetup(setup, true);
    return result.isEmpty();
}

float AudioCore::getCpuLoad() const
{
    return static_cast<float>(deviceManager->getCpuUsage());
}

// === Transport ===

void AudioCore::play()
{
    playing = true;
}

void AudioCore::pause()
{
    playing = false;
}

void AudioCore::stop()
{
    playing = false;
    playheadBeats = 0.0;
}

void AudioCore::setPlayheadBeats(double beats)
{
    playheadBeats = beats;
}

void AudioCore::setBpm(double newBpm)
{
    bpm = juce::jlimit(20.0, 999.0, newBpm);
    // Recalculate samples per beat
    if (currentSampleRate > 0) {
        samplesPerBeat = (60.0 / bpm.load()) * currentSampleRate;
    }
}

double AudioCore::getPlayheadSeconds() const
{
    return (playheadBeats.load() / bpm.load()) * 60.0;
}

void AudioCore::setLoopRegion(double startBeats, double endBeats)
{
    loopStartBeats = startBeats;
    loopEndBeats = endBeats;
}

void AudioCore::toggleLoop()
{
    looping = !looping.load();
}

void AudioCore::toggleRecord()
{
    recording = !recording.load();
}

// === Tracks ===

int AudioCore::addTrack()
{
    juce::ScopedLock lock(trackLock);
    auto track = std::make_unique<TrackProcessor>(nextTrackId);
    int id = nextTrackId++;
    tracks.push_back(std::move(track));
    return id;
}

void AudioCore::removeTrack(int trackId)
{
    juce::ScopedLock lock(trackLock);
    tracks.erase(
        std::remove_if(tracks.begin(), tracks.end(),
            [trackId](const auto& t) { return t->getId() == trackId; }),
        tracks.end());
}

TrackProcessor* AudioCore::getTrack(int trackId)
{
    juce::ScopedLock lock(trackLock);
    for (auto& track : tracks) {
        if (track->getId() == trackId)
            return track.get();
    }
    return nullptr;
}

int AudioCore::getTrackCount() const
{
    return static_cast<int>(tracks.size());
}

// === Plugin Host ===

void AudioCore::scanPlugins()
{
    pluginHost->scanPlugins();
}

bool AudioCore::loadPlugin(int trackId, int slot, const juce::String& pluginId)
{
    auto* track = getTrack(trackId);
    if (!track) return false;
    return track->loadPlugin(slot, *pluginHost, pluginId);
}

void AudioCore::unloadPlugin(int trackId, int slot)
{
    auto* track = getTrack(trackId);
    if (track) {
        track->unloadPlugin(slot);
    }
}

// === AudioSource ===

void AudioCore::prepareToPlay(int samplesPerBlockExpected, double sampleRate)
{
    currentSampleRate = static_cast<int>(sampleRate);
    currentBufferSize = samplesPerBlockExpected;
    samplesPerBeat = (60.0 / bpm.load()) * sampleRate;

    juce::ScopedLock lock(trackLock);
    for (auto& track : tracks) {
        track->prepareToPlay(sampleRate, samplesPerBlockExpected);
    }

    DBG("AudioCore prepared: " << sampleRate << " Hz, " << samplesPerBlockExpected << " samples");
}

void AudioCore::releaseResources()
{
    juce::ScopedLock lock(trackLock);
    for (auto& track : tracks) {
        track->releaseResources();
    }
}

void AudioCore::getNextAudioBlock(const juce::AudioSourceChannelInfo& bufferToFill)
{
    bufferToFill.clearActiveBufferRegion();

    if (!playing.load()) {
        return;
    }

    const int numSamples = bufferToFill.numSamples;
    auto* buffer = bufferToFill.buffer;

    // Process all tracks
    {
        juce::ScopedLock lock(trackLock);
        for (auto& track : tracks) {
            track->processBlock(*buffer, playheadBeats.load(), bpm.load(), currentSampleRate);
        }
    }

    // Advance playhead
    double beatsAdvanced = static_cast<double>(numSamples) / samplesPerBeat;
    double newPlayhead = playheadBeats.load() + beatsAdvanced;

    // Handle loop
    if (looping.load() && newPlayhead >= loopEndBeats) {
        newPlayhead = loopStartBeats + (newPlayhead - loopEndBeats);
    }

    playheadBeats = newPlayhead;
}

// === Change Listener ===

void AudioCore::changeListenerCallback(juce::ChangeBroadcaster* source)
{
    if (source == deviceManager.get()) {
        // Audio device settings changed
        auto* device = deviceManager->getCurrentAudioDevice();
        if (device) {
            currentSampleRate = static_cast<int>(device->getCurrentSampleRate());
            currentBufferSize = device->getCurrentBufferSizeSamples();
            DBG("Audio device changed: " << currentSampleRate << " Hz, " << currentBufferSize << " samples");
        }
    }
}

// === Timer (FFI Processing) ===

void AudioCore::timerCallback()
{
    processCommands();
    updateStateSnapshot();
}

void AudioCore::processCommands()
{
#ifdef RYSYN_FFI_AVAILABLE
    // Process up to 100 commands per frame
    for (int i = 0; i < 100; ++i) {
        char* cmdJson = rysyn_recv_command_json();
        if (cmdJson == nullptr) break;

        // Parse JSON command
        juce::String jsonStr(cmdJson);
        rysyn_free_string(cmdJson);

        auto result = juce::JSON::parse(jsonStr);
        if (result.isVoid()) {
            DBG("Failed to parse command JSON: " << jsonStr);
            continue;
        }

        // Commands are serialized as: {"CommandName": null} or {"CommandName": {...params...}}
        if (auto* obj = result.getDynamicObject()) {
            for (auto& prop : obj->getProperties()) {
                auto cmdName = prop.name.toString();
                auto& params = prop.value;

                if (cmdName == "Play") {
                    play();
                }
                else if (cmdName == "Pause") {
                    pause();
                }
                else if (cmdName == "Stop") {
                    stop();
                }
                else if (cmdName == "ToggleRecord") {
                    toggleRecord();
                }
                else if (cmdName == "ToggleLoop") {
                    toggleLoop();
                }
                else if (cmdName == "SetTempo") {
                    if (auto* p = params.getDynamicObject()) {
                        setBpm(p->getProperty("bpm"));
                    }
                }
                else if (cmdName == "SetPlayhead") {
                    if (auto* p = params.getDynamicObject()) {
                        setPlayheadBeats(p->getProperty("beats"));
                    }
                }
                else if (cmdName == "SetLoopRegion") {
                    if (auto* p = params.getDynamicObject()) {
                        setLoopRegion(p->getProperty("start_beats"), p->getProperty("end_beats"));
                    }
                }
                else if (cmdName == "CreateTrack") {
                    if (auto* p = params.getDynamicObject()) {
                        auto trackId = addTrack();
                        if (trackId >= 0 && trackId < static_cast<int>(tracks.size())) {
                            auto& track = tracks[trackId];
                            if (p->hasProperty("name")) {
                                track->setName(p->getProperty("name").toString());
                            }
                            if (p->hasProperty("is_midi")) {
                                // Store MIDI flag if needed later
                                // For now, all tracks are audio-capable but can receive MIDI plugins
                            }
                        }
                    }
                }
                else if (cmdName == "DeleteTrack") {
                    if (auto* p = params.getDynamicObject()) {
                        removeTrack((int)p->getProperty("track_id"));
                    }
                }
                else if (cmdName == "SetTrackVolume") {
                    if (auto* p = params.getDynamicObject()) {
                        if (auto* track = getTrack((int)p->getProperty("track_id"))) {
                            track->setVolume((float)p->getProperty("volume"));
                        }
                    }
                }
                else if (cmdName == "SetTrackPan") {
                    if (auto* p = params.getDynamicObject()) {
                        if (auto* track = getTrack((int)p->getProperty("track_id"))) {
                            track->setPan((float)p->getProperty("pan"));
                        }
                    }
                }
                else if (cmdName == "SetTrackMute") {
                    if (auto* p = params.getDynamicObject()) {
                        if (auto* track = getTrack((int)p->getProperty("track_id"))) {
                            track->setMuted((bool)p->getProperty("muted"));
                        }
                    }
                }
                else if (cmdName == "SetTrackSolo") {
                    if (auto* p = params.getDynamicObject()) {
                        if (auto* track = getTrack((int)p->getProperty("track_id"))) {
                            track->setSoloed((bool)p->getProperty("soloed"));
                        }
                    }
                }
                else if (cmdName == "RescanPlugins") {
                    scanPlugins();
                }
                else if (cmdName == "LoadPlugin") {
                    if (auto* p = params.getDynamicObject()) {
                        int trackId = (int)p->getProperty("track_id");
                        juce::String pluginId = p->getProperty("plugin_id").toString();
                        int slot = p->hasProperty("slot") ? (int)p->getProperty("slot") : 0;
                        loadPlugin(trackId, slot, pluginId);
                    }
                }
                else if (cmdName == "RemovePlugin") {
                    if (auto* p = params.getDynamicObject()) {
                        unloadPlugin((int)p->getProperty("track_id"), (int)p->getProperty("slot"));
                    }
                }
                else {
                    DBG("Unknown command: " << cmdName);
                }
            }
        }
    }
#endif
}

void AudioCore::updateStateSnapshot()
{
#ifdef RYSYN_FFI_AVAILABLE
    // Build state JSON matching Rust StateSnapshot structure
    juce::DynamicObject::Ptr state = new juce::DynamicObject();
    
    // Transport state
    juce::DynamicObject::Ptr transport = new juce::DynamicObject();
    transport->setProperty("is_playing", playing.load());
    transport->setProperty("is_recording", recording.load());
    transport->setProperty("is_looping", looping.load());
    transport->setProperty("playhead_beats", playheadBeats.load());
    transport->setProperty("playhead_seconds", getPlayheadSeconds());
    transport->setProperty("tempo", bpm.load());
    transport->setProperty("time_sig_num", 4);
    transport->setProperty("time_sig_denom", 4);
    transport->setProperty("loop_start_beats", loopStartBeats.load());
    transport->setProperty("loop_end_beats", loopEndBeats.load());
    state->setProperty("transport", juce::var(transport.get()));

    // Meter levels
    juce::DynamicObject::Ptr meters = new juce::DynamicObject();
    meters->setProperty("master_l", masterPeakL.load());
    meters->setProperty("master_r", masterPeakR.load());
    juce::Array<juce::var> trackLevelsL, trackLevelsR;
    {
        juce::ScopedLock lock(trackLock);
        for (auto& track : tracks) {
            trackLevelsL.add(track->getPeakL());
            trackLevelsR.add(track->getPeakR());
        }
    }
    meters->setProperty("track_levels_l", trackLevelsL);
    meters->setProperty("track_levels_r", trackLevelsR);
    state->setProperty("meters", juce::var(meters.get()));

    // Tracks
    juce::Array<juce::var> tracksArray;
    {
        juce::ScopedLock lock(trackLock);
        for (auto& track : tracks) {
            juce::DynamicObject::Ptr trackObj = new juce::DynamicObject();
            trackObj->setProperty("id", track->getId());
            trackObj->setProperty("name", track->getName());
            trackObj->setProperty("color", (int)0xFF808080);
            trackObj->setProperty("volume", track->getVolume());
            trackObj->setProperty("pan", track->getPan());
            trackObj->setProperty("is_muted", track->isMuted());
            trackObj->setProperty("is_soloed", track->isSoloed());
            trackObj->setProperty("is_armed", false);
            trackObj->setProperty("is_midi", false);
            trackObj->setProperty("is_selected", track->isSelected());
            trackObj->setProperty("input_name", "No Input");
            trackObj->setProperty("output_name", "Master");
            
            // Plugin slots
            juce::Array<juce::var> pluginsArray;
            for (int i = 0; i < 8; ++i) {
                auto* plugin = track->getPlugin(i);
                if (plugin) {
                    pluginsArray.add(plugin->getName());
                } else {
                    pluginsArray.add(juce::var());
                }
            }
            trackObj->setProperty("plugins", pluginsArray);
            
            tracksArray.add(juce::var(trackObj.get()));
        }
    }
    state->setProperty("tracks", tracksArray);

    // Clips (empty for now)
    state->setProperty("clips", juce::Array<juce::var>());

    // Available plugins
    juce::Array<juce::var> pluginsArray;
    int pluginCount = pluginHost->getPluginCount();
    for (int i = 0; i < pluginCount; ++i) {
        juce::DynamicObject::Ptr pluginObj = new juce::DynamicObject();
        pluginObj->setProperty("id", pluginHost->getPluginId(i));
        pluginObj->setProperty("name", pluginHost->getPluginName(i));
        pluginObj->setProperty("manufacturer", pluginHost->getPluginVendor(i));
        pluginObj->setProperty("format", "VST3");
        pluginObj->setProperty("path", "");
        pluginObj->setProperty("is_instrument", pluginHost->isPluginInstrument(i));
        pluginsArray.add(juce::var(pluginObj.get()));
    }
    state->setProperty("available_plugins", pluginsArray);

    // System info
    state->setProperty("master_volume", masterVolume.load());
    state->setProperty("cpu_load", getCpuLoad());
    state->setProperty("sample_rate", currentSampleRate);
    state->setProperty("buffer_size", currentBufferSize);
    state->setProperty("audio_device_name", deviceManager->getCurrentAudioDeviceType());
    state->setProperty("project_name", "Untitled");
    state->setProperty("is_modified", false);

    // Serialize and send to Rust
    juce::String jsonStr = juce::JSON::toString(juce::var(state.get()), true);
    rysyn_update_state_json(jsonStr.toRawUTF8());
#endif
}

} // namespace rysyn

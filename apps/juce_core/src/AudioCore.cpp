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

        // Parse and handle command
        juce::String jsonStr(cmdJson);
        rysyn_free_string(cmdJson);

        // TODO: Parse JSON and dispatch command
        // For now, just log
        DBG("Received command: " << jsonStr);
    }
#endif
}

void AudioCore::updateStateSnapshot()
{
#ifdef RYSYN_FFI_AVAILABLE
    // Build state JSON
    juce::DynamicObject::Ptr state = new juce::DynamicObject();
    
    // Transport
    juce::DynamicObject::Ptr transport = new juce::DynamicObject();
    transport->setProperty("is_playing", playing.load());
    transport->setProperty("is_recording", recording.load());
    transport->setProperty("is_looping", looping.load());
    transport->setProperty("playhead_beats", playheadBeats.load());
    transport->setProperty("playhead_seconds", getPlayheadSeconds());
    transport->setProperty("bpm", bpm.load());
    state->setProperty("transport", juce::var(transport.get()));

    // Tracks
    juce::Array<juce::var> tracksArray;
    {
        juce::ScopedLock lock(trackLock);
        for (auto& track : tracks) {
            juce::DynamicObject::Ptr trackObj = new juce::DynamicObject();
            trackObj->setProperty("id", track->getId());
            trackObj->setProperty("name", track->getName());
            trackObj->setProperty("volume", track->getVolume());
            trackObj->setProperty("pan", track->getPan());
            trackObj->setProperty("mute", track->isMuted());
            trackObj->setProperty("solo", track->isSoloed());
            tracksArray.add(juce::var(trackObj.get()));
        }
    }
    state->setProperty("tracks", tracksArray);

    // System
    state->setProperty("cpu_load", getCpuLoad());
    state->setProperty("sample_rate", currentSampleRate);
    state->setProperty("buffer_size", currentBufferSize);

    // Serialize and send
    juce::String jsonStr = juce::JSON::toString(juce::var(state.get()));
    rysyn_update_state_json(jsonStr.toRawUTF8());
#endif
}

} // namespace rysyn

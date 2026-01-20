#pragma once

#include <JuceHeader.h>
#include <memory>
#include <vector>
#include <functional>

// Forward declare FFI functions if available
#ifdef RYSYN_FFI_AVAILABLE
extern "C" {
    bool rysyn_init();
    char* rysyn_get_state_json();
    bool rysyn_update_state_json(const char* json);
    char* rysyn_recv_command_json();
    void rysyn_free_string(char* s);
    bool rysyn_transport_play();
    bool rysyn_transport_pause();
    bool rysyn_transport_stop();
    bool rysyn_transport_set_bpm(double bpm);
}
#endif

namespace rysyn {

class TrackProcessor;
class PluginHost;

/**
 * Main Audio Core
 * 
 * Manages:
 * - Audio device I/O
 * - Track graph processing
 * - VST3/AU plugin hosting
 * - Transport synchronization
 * - Communication with Rust/egui via FFI
 */
class AudioCore : public juce::AudioSource,
                  public juce::ChangeListener,
                  private juce::Timer
{
public:
    AudioCore();
    ~AudioCore() override;

    // === Lifecycle ===
    bool initialize();
    void shutdown();

    // === Audio Device ===
    juce::StringArray getAvailableDevices() const;
    bool setAudioDevice(const juce::String& deviceName);
    int getSampleRate() const { return currentSampleRate; }
    int getBufferSize() const { return currentBufferSize; }
    float getCpuLoad() const;

    // === Transport ===
    void play();
    void pause();
    void stop();
    void setPlayheadBeats(double beats);
    void setBpm(double bpm);
    void setLoopRegion(double startBeats, double endBeats);
    void toggleLoop();
    void toggleRecord();

    double getPlayheadBeats() const { return playheadBeats; }
    double getPlayheadSeconds() const;
    double getBpm() const { return bpm; }
    bool isPlaying() const { return playing; }
    bool isRecording() const { return recording; }
    bool isLooping() const { return looping; }

    // Tracks
    int addTrack();
    void removeTrack(int trackId);
    TrackProcessor* getTrack(int trackId);
    int getTrackCount() const;

    // Plugin Host
    PluginHost& getPluginHost() { return *pluginHost; }
    void scanPlugins();
    bool loadPlugin(int trackId, int slot, const juce::String& pluginId);
    void unloadPlugin(int trackId, int slot);

    // AudioSource interface 
    void prepareToPlay(int samplesPerBlockExpected, double sampleRate) override;
    void releaseResources() override;
    void getNextAudioBlock(const juce::AudioSourceChannelInfo& bufferToFill) override;

    // ffi communication
    void processCommands();  // Called on message thread
    void updateStateSnapshot();  // Push state to Rust

private:
    void changeListenerCallback(juce::ChangeBroadcaster* source) override;
    void timerCallback() override;

    // Audio
    std::unique_ptr<juce::AudioDeviceManager> deviceManager;
    std::unique_ptr<juce::AudioSourcePlayer> sourcePlayer;
    std::unique_ptr<PluginHost> pluginHost;

    // Tracks
    std::vector<std::unique_ptr<TrackProcessor>> tracks;
    juce::CriticalSection trackLock;
    int nextTrackId = 1;

    // Transport
    std::atomic<bool> playing{false};
    std::atomic<bool> recording{false};
    std::atomic<bool> looping{false};
    std::atomic<double> playheadBeats{0.0};
    std::atomic<double> bpm{120.0};
    std::atomic<double> loopStartBeats{0.0};
    std::atomic<double> loopEndBeats{16.0};

    // Master metering
    std::atomic<float> masterPeakL{0.0f};
    std::atomic<float> masterPeakR{0.0f};
    std::atomic<float> masterVolume{1.0f};

    // Audio state
    int currentSampleRate = 44100;
    int currentBufferSize = 512;
    double samplesPerBeat = 0.0;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(AudioCore)
};

} // namespace rysyn

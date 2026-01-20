#pragma once

#include <JuceHeader.h>
#include <memory>
#include <vector>

namespace rysyn {

class PluginHost;

/**
 * Single track processor
 * 
 * Handles:
 * - Audio/MIDI clips
 * - Plugin chain
 * - Volume/Pan/Mute/Solo
 * - Metering
 */
class TrackProcessor
{
public:
    explicit TrackProcessor(int id);
    ~TrackProcessor();

    // Identity
    int getId() const { return trackId; }
    juce::String getName() const { return name; }
    void setName(const juce::String& newName) { name = newName; }

    // Mix controls
    float getVolume() const { return volume; }
    void setVolume(float v) { volume = juce::jlimit(0.0f, 2.0f, v); }
    float getPan() const { return pan; }
    void setPan(float p) { pan = juce::jlimit(-1.0f, 1.0f, p); }
    bool isMuted() const { return muted; }
    void setMuted(bool m) { muted = m; }
    bool isSoloed() const { return soloed; }
    void setSoloed(bool s) { soloed = s; }
    bool isArmed() const { return armed; }
    void setArmed(bool a) { armed = a; }

    // Color (RGBA)
    uint32_t getColor() const { return color; }
    void setColor(uint32_t c) { color = c; }

    // Metering
    float getPeakL() const { return peakL; }
    float getPeakR() const { return peakR; }

    // Selection
    bool isSelected() const { return selected; }
    void setSelected(bool s) { selected = s; }

    // Audio Clips
    struct Clip
    {
        juce::String name;
        juce::AudioBuffer<float> audioData;
        double startBeats = 0.0;      // When clip starts in timeline
        double clipStartBeats = 0.0;  // Where to start playing in clip
        double lengthBeats = 0.0;     // Duration in beats
        bool enabled = true;
    };

    void addClip(const juce::String& name, const juce::AudioBuffer<float>& audioData,
                 double startBeats, double lengthBeats);
    void removeClip(int index);
    const std::vector<Clip>& getClips() const { return clips; }
    void clearClips() { clips.clear(); }

    // Processing
    void prepareToPlay(double sampleRate, int blockSize);
    void releaseResources();
    void processBlock(juce::AudioBuffer<float>& buffer, double playheadBeats, double bpm, int sampleRate);

    // Plugins
    bool loadPlugin(int slot, PluginHost& host, const juce::String& pluginId);
    void unloadPlugin(int slot);
    int getPluginCount() const;
    juce::AudioPluginInstance* getPlugin(int slot) const {
        if (slot >= 0 && slot < static_cast<int>(plugins.size())) {
            return plugins[static_cast<size_t>(slot)].get();
        }
        return nullptr;
    }

private:
    int trackId;
    juce::String name;

    // Mix
    float volume = 1.0f;
    float pan = 0.0f;
    bool muted = false;
    bool soloed = false;
    bool armed = false;
    bool selected = false;
    uint32_t color = 0x4A90D9FF; // Default blue

    // Metering
    float peakL = 0.0f;
    float peakR = 0.0f;

    // Audio clips
    std::vector<Clip> clips;

    // Plugin slots (max 8)
    std::array<std::unique_ptr<juce::AudioPluginInstance>, 8> plugins;

    // Temporary buffer for plugin processing
    juce::AudioBuffer<float> tempBuffer;

    // Tracking clip playback position per sample
    double currentClipPlayheadBeats = 0.0;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(TrackProcessor)
};

} // namespace rysyn

#pragma once

#include <JuceHeader.h>
#include <atomic>

namespace rysyn {

/**
 * Transport state machine
 * 
 * Thread-safe access to playback state.
 * Used by both audio thread (read) and message thread (write).
 */
class Transport
{
public:
    Transport();

    // State
    void play();
    void pause();
    void stop();
    void togglePlayPause();
    void toggleRecord();
    void toggleLoop();

    bool isPlaying() const { return playing.load(std::memory_order_acquire); }
    bool isRecording() const { return recording.load(std::memory_order_acquire); }
    bool isLooping() const { return looping.load(std::memory_order_acquire); }

    // Position
    void setPlayheadBeats(double beats);
    double getPlayheadBeats() const { return playheadBeats.load(std::memory_order_acquire); }
    void advancePlayhead(double beats);

    // Tempo
    void setBpm(double bpm);
    double getBpm() const { return bpm.load(std::memory_order_acquire); }
    void setTimeSignature(int num, int denom);
    int getTimeSignatureNum() const { return timeSigNum.load(std::memory_order_acquire); }
    int getTimeSignatureDenom() const { return timeSigDenom.load(std::memory_order_acquire); }

    // Loop region
    void setLoopRegion(double startBeats, double endBeats);
    double getLoopStartBeats() const { return loopStartBeats.load(std::memory_order_acquire); }
    double getLoopEndBeats() const { return loopEndBeats.load(std::memory_order_acquire); }

    // Conversion helpers
    double beatsToSeconds(double beats) const;
    double secondsToBeats(double seconds) const;
    double beatsToSamples(double beats, double sampleRate) const;
    double samplesToBeats(double samples, double sampleRate) const;

    // Bar/beat helpers
    int getCurrentBar() const;
    double getCurrentBeatInBar() const;
    juce::String getPositionString() const;

private:
    std::atomic<bool> playing{false};
    std::atomic<bool> recording{false};
    std::atomic<bool> looping{false};

    std::atomic<double> playheadBeats{0.0};
    std::atomic<double> bpm{120.0};
    std::atomic<int> timeSigNum{4};
    std::atomic<int> timeSigDenom{4};

    std::atomic<double> loopStartBeats{0.0};
    std::atomic<double> loopEndBeats{16.0};

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(Transport)
};

} // namespace rysyn

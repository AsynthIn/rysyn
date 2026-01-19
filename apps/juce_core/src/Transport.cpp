#include "Transport.h"

namespace rysyn {

Transport::Transport()
{
}

void Transport::play()
{
    playing.store(true, std::memory_order_release);
}

void Transport::pause()
{
    playing.store(false, std::memory_order_release);
}

void Transport::stop()
{
    playing.store(false, std::memory_order_release);
    playheadBeats.store(0.0, std::memory_order_release);
}

void Transport::togglePlayPause()
{
    playing.store(!playing.load(std::memory_order_acquire), std::memory_order_release);
}

void Transport::toggleRecord()
{
    recording.store(!recording.load(std::memory_order_acquire), std::memory_order_release);
}

void Transport::toggleLoop()
{
    looping.store(!looping.load(std::memory_order_acquire), std::memory_order_release);
}

void Transport::setPlayheadBeats(double beats)
{
    playheadBeats.store(juce::jmax(0.0, beats), std::memory_order_release);
}

void Transport::advancePlayhead(double beats)
{
    double current = playheadBeats.load(std::memory_order_acquire);
    double newPos = current + beats;

    // Handle loop
    if (looping.load(std::memory_order_acquire)) {
        double loopEnd = loopEndBeats.load(std::memory_order_acquire);
        double loopStart = loopStartBeats.load(std::memory_order_acquire);

        if (newPos >= loopEnd) {
            newPos = loopStart + (newPos - loopEnd);
        }
    }

    playheadBeats.store(newPos, std::memory_order_release);
}

void Transport::setBpm(double newBpm)
{
    bpm.store(juce::jlimit(20.0, 999.0, newBpm), std::memory_order_release);
}

void Transport::setTimeSignature(int num, int denom)
{
    timeSigNum.store(juce::jlimit(1, 16, num), std::memory_order_release);
    timeSigDenom.store(juce::jlimit(1, 16, denom), std::memory_order_release);
}

void Transport::setLoopRegion(double startBeats, double endBeats)
{
    if (startBeats < endBeats) {
        loopStartBeats.store(startBeats, std::memory_order_release);
        loopEndBeats.store(endBeats, std::memory_order_release);
    }
}

double Transport::beatsToSeconds(double beats) const
{
    return (beats / bpm.load(std::memory_order_acquire)) * 60.0;
}

double Transport::secondsToBeats(double seconds) const
{
    return (seconds * bpm.load(std::memory_order_acquire)) / 60.0;
}

double Transport::beatsToSamples(double beats, double sampleRate) const
{
    return beatsToSeconds(beats) * sampleRate;
}

double Transport::samplesToBeats(double samples, double sampleRate) const
{
    return secondsToBeats(samples / sampleRate);
}

int Transport::getCurrentBar() const
{
    double beats = playheadBeats.load(std::memory_order_acquire);
    int beatsPerBar = timeSigNum.load(std::memory_order_acquire);
    return static_cast<int>(beats / beatsPerBar) + 1;
}

double Transport::getCurrentBeatInBar() const
{
    double beats = playheadBeats.load(std::memory_order_acquire);
    int beatsPerBar = timeSigNum.load(std::memory_order_acquire);
    return std::fmod(beats, static_cast<double>(beatsPerBar)) + 1.0;
}

juce::String Transport::getPositionString() const
{
    int bar = getCurrentBar();
    double beatInBar = getCurrentBeatInBar();
    int wholeBeat = static_cast<int>(beatInBar);
    int ticks = static_cast<int>((beatInBar - wholeBeat) * 960); // 960 ticks per beat

    return juce::String::formatted("%d.%d.%03d", bar, wholeBeat, ticks);
}

} // namespace rysyn

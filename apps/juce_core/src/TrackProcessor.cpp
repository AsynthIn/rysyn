#include "TrackProcessor.h"
#include "PluginHost.h"

namespace rysyn {

TrackProcessor::TrackProcessor(int id)
    : trackId(id)
    , name(juce::String("Track ") + juce::String(id))
{
}

TrackProcessor::~TrackProcessor()
{
    releaseResources();
}

void TrackProcessor::prepareToPlay(double sampleRate, int blockSize)
{
    tempBuffer.setSize(2, blockSize);

    for (auto& plugin : plugins) {
        if (plugin) {
            plugin->setPlayConfigDetails(2, 2, sampleRate, blockSize);
            plugin->prepareToPlay(sampleRate, blockSize);
        }
    }
}

void TrackProcessor::releaseResources()
{
    for (auto& plugin : plugins) {
        if (plugin) {
            plugin->releaseResources();
        }
    }
    tempBuffer.setSize(0, 0);
}

void TrackProcessor::processBlock(juce::AudioBuffer<float>& buffer, 
                                   double playheadBeats, 
                                   double bpm, 
                                   int sampleRate)
{
    if (muted) {
        return;
    }

    const int numSamples = buffer.getNumSamples();
    const int numChannels = buffer.getNumChannels();

    // TODO: Process audio clips based on playhead position

    // Process plugin chain
    juce::MidiBuffer midiBuffer;
    for (auto& plugin : plugins) {
        if (plugin && !plugin->isSuspended()) {
            plugin->processBlock(buffer, midiBuffer);
        }
    }

    // Apply volume and pan
    if (numChannels >= 2) {
        // Simple pan law
        float leftGain = volume * (pan <= 0.0f ? 1.0f : 1.0f - pan);
        float rightGain = volume * (pan >= 0.0f ? 1.0f : 1.0f + pan);

        buffer.applyGain(0, 0, numSamples, leftGain);
        buffer.applyGain(1, 0, numSamples, rightGain);

        // Update meters
        peakL = buffer.getMagnitude(0, 0, numSamples);
        peakR = buffer.getMagnitude(1, 0, numSamples);
    } else if (numChannels == 1) {
        buffer.applyGain(volume);
        peakL = peakR = buffer.getMagnitude(0, 0, numSamples);
    }
}

bool TrackProcessor::loadPlugin(int slot, PluginHost& host, const juce::String& pluginId)
{
    if (slot < 0 || slot >= static_cast<int>(plugins.size())) {
        return false;
    }

    // Unload existing
    unloadPlugin(slot);

    // Load new plugin
    auto plugin = host.createPluginInstance(pluginId);
    if (!plugin) {
        return false;
    }

    plugins[slot] = std::move(plugin);
    DBG("Loaded plugin " << pluginId << " to track " << trackId << " slot " << slot);
    return true;
}

void TrackProcessor::unloadPlugin(int slot)
{
    if (slot >= 0 && slot < static_cast<int>(plugins.size()) && plugins[slot]) {
        plugins[slot]->releaseResources();
        plugins[slot].reset();
    }
}

int TrackProcessor::getPluginCount() const
{
    int count = 0;
    for (const auto& plugin : plugins) {
        if (plugin) count++;
    }
    return count;
}

} // namespace rysyn

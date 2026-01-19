#pragma once

#include <JuceHeader.h>
#include <memory>
#include <vector>

namespace rysyn {

/**
 * Plugin scanner and factory
 * 
 * Supports:
 * - VST3 plugins
 * - AU plugins (macOS)
 * - LADSPA plugins (Linux)
 */
class PluginHost
{
public:
    PluginHost();
    ~PluginHost();

    // Scanning
    void scanPlugins();
    void addCustomSearchPath(const juce::String& path);
    bool isScanning() const { return scanning; }

    // Plugin list
    int getPluginCount() const;
    juce::String getPluginName(int index) const;
    juce::String getPluginId(int index) const;
    juce::String getPluginVendor(int index) const;
    juce::String getPluginCategory(int index) const;
    bool isPluginInstrument(int index) const;

    // Factory
    std::unique_ptr<juce::AudioPluginInstance> createPluginInstance(const juce::String& pluginId);

    // Plugin list access
    const juce::KnownPluginList& getKnownPluginList() const { return knownPlugins; }

private:
    juce::AudioPluginFormatManager formatManager;
    juce::KnownPluginList knownPlugins;
    std::vector<juce::String> customSearchPaths;
    std::atomic<bool> scanning{false};

    // Background scanner
    std::unique_ptr<juce::PluginDirectoryScanner> scanner;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(PluginHost)
};

} // namespace rysyn

#include "PluginHost.h"

namespace rysyn {

PluginHost::PluginHost()
{
    // Register plugin formats
    formatManager.addDefaultFormats();  // VST3, AU, etc.
}

PluginHost::~PluginHost()
{
    // Ensure any scanning is stopped
    scanner.reset();
}

void PluginHost::addCustomSearchPath(const juce::String& path)
{
    customSearchPaths.push_back(path);
}

void PluginHost::scanPlugins()
{
    if (scanning.load()) {
        DBG("Already scanning plugins");
        return;
    }

    scanning = true;
    DBG("Starting plugin scan...");

    // Get default VST3 paths
    juce::FileSearchPath searchPath;
    
    // Standard VST3 locations
#if JUCE_WINDOWS
    searchPath.add(juce::File::getSpecialLocation(juce::File::globalApplicationsDirectory)
        .getChildFile("Common Files/VST3"));
#elif JUCE_MAC
    searchPath.add(juce::File("/Library/Audio/Plug-Ins/VST3"));
    searchPath.add(juce::File("~/Library/Audio/Plug-Ins/VST3"));
#elif JUCE_LINUX
    searchPath.add(juce::File("/usr/lib/vst3"));
    searchPath.add(juce::File("/usr/local/lib/vst3"));
    searchPath.add(juce::File::getSpecialLocation(juce::File::userHomeDirectory).getChildFile(".vst3"));
#endif

    // Add custom paths
    for (const auto& path : customSearchPaths) {
        searchPath.add(juce::File(path));
    }

    // Scan for each format
    for (int i = 0; i < formatManager.getNumFormats(); ++i) {
        auto* format = formatManager.getFormat(i);
        
        juce::PluginDirectoryScanner localScanner(
            knownPlugins,
            *format,
            searchPath,
            true,  // recursive
            juce::File()  // dead plugins file
        );

        juce::String pluginName;
        while (localScanner.scanNextFile(true, pluginName)) {
            DBG("Scanned: " << pluginName);
        }
    }

    DBG("Plugin scan complete. Found " << knownPlugins.getNumTypes() << " plugins.");
    scanning = false;
}

int PluginHost::getPluginCount() const
{
    return knownPlugins.getNumTypes();
}

juce::String PluginHost::getPluginName(int index) const
{
    if (index >= 0 && index < knownPlugins.getNumTypes()) {
        return knownPlugins.getTypes()[index].name;
    }
    return {};
}

juce::String PluginHost::getPluginId(int index) const
{
    if (index >= 0 && index < knownPlugins.getNumTypes()) {
        return knownPlugins.getTypes()[index].createIdentifierString();
    }
    return {};
}

juce::String PluginHost::getPluginVendor(int index) const
{
    if (index >= 0 && index < knownPlugins.getNumTypes()) {
        return knownPlugins.getTypes()[index].manufacturerName;
    }
    return {};
}

juce::String PluginHost::getPluginCategory(int index) const
{
    if (index >= 0 && index < knownPlugins.getNumTypes()) {
        return knownPlugins.getTypes()[index].category;
    }
    return {};
}

bool PluginHost::isPluginInstrument(int index) const
{
    if (index >= 0 && index < knownPlugins.getNumTypes()) {
        return knownPlugins.getTypes()[index].isInstrument;
    }
    return false;
}

std::unique_ptr<juce::AudioPluginInstance> PluginHost::createPluginInstance(const juce::String& pluginId)
{
    // Find the plugin description
    const juce::PluginDescription* desc = nullptr;
    for (const auto& type : knownPlugins.getTypes()) {
        if (type.createIdentifierString() == pluginId) {
            desc = &type;
            break;
        }
    }

    if (!desc) {
        DBG("Plugin not found: " << pluginId);
        return nullptr;
    }

    juce::String error;
    auto instance = formatManager.createPluginInstance(
        *desc,
        44100.0,  // Default sample rate, will be updated
        512,      // Default block size
        error
    );

    if (!instance) {
        DBG("Failed to create plugin instance: " << error);
    }

    return instance;
}

} // namespace rysyn

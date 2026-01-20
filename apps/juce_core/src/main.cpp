/**
 * Rysyn DAW - JUCE Audio Core Test
 * 
 * Standalone executable for testing audio engine without egui.
 */

#include <JuceHeader.h>
#include "AudioCore.h"
#include "PluginHost.h"

class RysynAudioTestApp : public juce::JUCEApplication
{
public:
    RysynAudioTestApp() {}

    const juce::String getApplicationName() override { return "Rysyn Audio Test"; }
    const juce::String getApplicationVersion() override { return "0.1.0"; }

    void initialise(const juce::String& commandLine) override
    {
        juce::ignoreUnused(commandLine);
        
        audioCore = std::make_unique<rysyn::AudioCore>();
        
        if (audioCore->initialize()) {
            std::cout << "Audio Core initialized successfully!\n";
            std::cout << "Sample Rate: " << audioCore->getSampleRate() << " Hz\n";
            std::cout << "Buffer Size: " << audioCore->getBufferSize() << " samples\n";
            
            // Add a test track
            int trackId = audioCore->addTrack();
            std::cout << "Added track with ID: " << trackId << "\n";

            // Scan plugins
            std::cout << "Scanning for VST3 plugins...\n";
            audioCore->scanPlugins();
            
            auto& pluginHost = audioCore->getPluginHost();
            int pluginCount = pluginHost.getPluginCount();
            std::cout << "Found " << pluginCount << " plugins:\n";
            
            for (int i = 0; i < std::min(pluginCount, 20); ++i) {
                std::cout << "  - " << pluginHost.getPluginName(i).toStdString() 
                          << " (" << pluginHost.getPluginVendor(i).toStdString() << ")\n";
            }
            if (pluginCount > 20) {
                std::cout << "  ... and " << (pluginCount - 20) << " more\n";
            }

            // Start playback test
            std::cout << "\nStarting playback test (BPM: 120)...\n";
            audioCore->setBpm(120.0);
            audioCore->play();

            // Run for a few seconds using Timer
            juce::Time::waitForMillisecondCounter(juce::Time::getMillisecondCounter() + 3000);

            audioCore->stop();
            std::cout << "Playback stopped.\n";

        } else {
            std::cerr << "Failed to initialize Audio Core!\n";
        }

        quit();
    }

    void shutdown() override
    {
        if (audioCore) {
            audioCore->shutdown();
            audioCore.reset();
        }
    }

private:
    std::unique_ptr<rysyn::AudioCore> audioCore;
};

// Main entry point
START_JUCE_APPLICATION(RysynAudioTestApp)

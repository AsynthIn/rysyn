#include "AudioLoader.h"

namespace rysyn {

juce::String AudioLoader::lastError;

juce::AudioBuffer<float> AudioLoader::loadAudioFile(const juce::String& filePath)
{
    juce::File file(filePath);
    
    if (!file.exists()) {
        lastError = "File not found: " + filePath;
        DBG(lastError);
        return {};
    }

    // Use JUCE's AudioFormatManager to load the file
    juce::AudioFormatManager formatManager;
    formatManager.registerBasicFormats();

    auto reader = std::unique_ptr<juce::AudioFormatReader>(
        formatManager.createReaderFor(file)
    );

    if (!reader) {
        lastError = "Failed to open audio file: " + filePath;
        DBG(lastError);
        return {};
    }

    // Read audio data into buffer
    juce::AudioBuffer<float> buffer(static_cast<int>(reader->numChannels),
                                    static_cast<int>(reader->lengthInSamples));
    
    if (!reader->read(&buffer, 0, static_cast<int>(reader->lengthInSamples), 0, true, true)) {
        lastError = "Failed to read audio data from: " + filePath;
        DBG(lastError);
        return {};
    }

    lastError = "";
    DBG("Loaded audio file: " << filePath << " (" << buffer.getNumChannels() 
        << " channels, " << buffer.getNumSamples() << " samples, "
        << reader->sampleRate << " Hz)");
    
    return buffer;
}

juce::String AudioLoader::getLastError()
{
    return lastError;
}

} // namespace rysyn

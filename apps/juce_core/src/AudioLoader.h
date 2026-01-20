#pragma once

#include <JuceHeader.h>
#include <memory>

namespace rysyn {

/**
 * Audio File Loader
 * 
 * Loads WAV/AIFF/FLAC files into memory buffers
 */
class AudioLoader
{
public:
    /**
     * Load an audio file from disk
     * 
     * @param filePath Path to the audio file
     * @return AudioBuffer containing the loaded audio, or empty buffer if failed
     */
    static juce::AudioBuffer<float> loadAudioFile(const juce::String& filePath);

    /**
     * Get error message from last load attempt
     */
    static juce::String getLastError();

private:
    static juce::String lastError;
};

} // namespace rysyn

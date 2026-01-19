import wave
import math
import struct

sample_rate = 44100
duration = 2.0
frequency = 440.0

num_samples = int(sample_rate * duration)

with wave.open("test.wav", "w") as wav_file:
    wav_file.setnchannels(1)
    wav_file.setsampwidth(2)
    wav_file.setframerate(sample_rate)
    
    for i in range(num_samples):
        value = int(32767.0 * math.sin(2.0 * math.pi * frequency * i / sample_rate))
        data = struct.pack('<h', value)
        wav_file.writeframes(data)

print("Generated test.wav")

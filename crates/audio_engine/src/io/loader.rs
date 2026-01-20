use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct AudioLoader;

impl AudioLoader {
    pub fn load_file(path: &str) -> Result<Vec<f32>, String> {
        let src = File::open(Path::new(path)).map_err(|e| e.to_string())?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let hint = Hint::new();
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| e.to_string())?;

        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("no supported audio tracks")?;

        let dec_opts: DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| e.to_string())?;

        let track_id = track.id;
        let mut samples: Vec<f32> = Vec::new();

        // Decode loop
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(Error::IoError(_)) => break, // End of stream
                Err(Error::ResetRequired) => {
                    // The track list has been changed. Re-examine it and create a new decoder instance.
                    // This typically happens with streaming formats that can change tracks mid-stream.
                    // For now, we'll try to continue by getting the track again and creating a new decoder.
                    match format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL) {
                        Some(new_track) => {
                            let new_dec_opts: DecoderOptions = Default::default();
                            match symphonia::default::get_codecs()
                                .make(&new_track.codec_params, &new_dec_opts)
                            {
                                Ok(new_decoder) => {
                                    decoder = new_decoder;
                                    continue;
                                }
                                Err(e) => return Err(format!("Failed to recreate decoder: {}", e)),
                            }
                        }
                        None => return Err("No valid audio track found after reset".to_string()),
                    }
                }
                Err(err) => return Err(err.to_string()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // For MVP: Monofier / Resampler placeholder. 
                    // We assume f32 output for internal engine processing.
                    // If stereo, we just take the first channel or mix them roughly for now.
                    // Ideally we keep channel data, but for "Hello Files", let's flatten to mono or interleaved.
                    
                    let spec = *decoded.spec();
                    let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                    sample_buf.copy_interleaved_ref(decoded);

                    // Interleaved samples
                    samples.extend_from_slice(sample_buf.samples()); 
                }
                Err(Error::IoError(_)) => break,
                Err(Error::DecodeError(_)) => (), // Ignore decode errors for now
                Err(err) => return Err(err.to_string()),
            }
        }

        Ok(samples)
    }
}

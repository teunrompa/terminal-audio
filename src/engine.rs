use cpal::{
    Device, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::mixer::Mixer;

pub struct AudioEngine {
    sample_rate: f32,
    state: AudioEngineState,
    stream_config: SupportedStreamConfig,
    channels: usize,
    device: Device,
    mixer: Mixer,
}

#[derive(PartialEq, Clone)]
enum AudioEngineState {
    Playing,
    Stopping,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device found")?;

        let config = device.default_output_config()?;
        let channels = config.channels() as usize;
        let sample_rate = config.sample_rate() as f32;

        Ok(AudioEngine {
            sample_rate,
            state: AudioEngineState::Stopping,
            device,
            stream_config: config,
            channels,
            mixer: Mixer::new(),
        })
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn process(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sample_clock = 0f32;
        let frequency = 440.0; // A4 note
        let sample_rate = self.sample_rate;
        let channels = self.channels;
        let stream_config = self.stream_config.clone();

        let stream = self.device.build_output_stream(
            &stream_config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let value =
                        (sample_clock * frequency * 2.0 * std::f32::consts::PI / sample_rate).sin();
                    sample_clock = (sample_clock + 1.0) % sample_rate;

                    // Write to all channels
                    for sample in frame.iter_mut() {
                        *sample = value * 0.3; // Reduced volume
                    }
                }
            },
            |err| eprintln!("Output error: {}", err),
            None,
        )?;

        stream.play();

        Ok(())
    }
}

use std::sync::{Arc, Mutex};

use cpal::{
    Device, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossterm::event::{KeyCode, KeyEvent};

use crate::mixer::Mixer;

pub struct AudioEngine {
    sample_rate: f32,
    state: Arc<Mutex<AudioEngineState>>,
    channels: u16,
    mixer: Arc<Mutex<Mixer>>,
    stream: Option<Stream>,
    device: Device,
}

#[derive(PartialEq, Clone, Copy)]
pub enum AudioEngineState {
    Playing,
    Stopped,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device found")?;

        let config = device.default_output_config()?;
        let channels = config.channels();
        let sample_rate = config.sample_rate() as f32;

        Ok(AudioEngine {
            sample_rate,
            state: Arc::new(Mutex::new(AudioEngineState::Stopped)),
            channels,
            mixer: Arc::new(Mutex::new(Mixer::new(sample_rate, 140.0))),
            stream: None,
            device,
        })
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //Set the state
        *self.state.lock().unwrap() = AudioEngineState::Playing;
        let mixer = Arc::clone(&self.mixer);
        let state = Arc::clone(&self.state);
        let chanels = self.channels as usize;

        let config = StreamConfig {
            channels: self.channels,
            sample_rate: self.sample_rate as u32,
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = self.device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if *state.lock().unwrap() == AudioEngineState::Stopped {
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }

                    return;
                }

                let num_frames = data.len() / chanels;

                let mix = {
                    let mut mixer_gaurd = mixer.lock().unwrap();
                    mixer_gaurd.process_block(num_frames)
                };

                for (frame_idx, frame) in data.chunks_mut(chanels).enumerate() {
                    let sample = mix.get(frame_idx).copied().unwrap_or(0.0);

                    for ch in frame.iter_mut() {
                        *ch = sample;
                    }
                }
            },
            |err| eprintln!("Audio error {}", err),
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) {
        *self.state.lock().unwrap() = AudioEngineState::Stopped;
        self.stream = None;
    }

    pub fn toggle_playback(&mut self) {
        let is_playing = *self.state.lock().unwrap() == AudioEngineState::Playing;

        if is_playing {
            self.stop();
        } else if let Err(e) = self.start() {
            eprintln!("Failed to start audio {}", e);
        }
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.handle_keyboard_input(key_event);
        }

        match key_event.code {
            KeyCode::Char(' ') => self.toggle_playback(),
            KeyCode::Char('w') => todo!(), //TODO: implement next window
            _ => {}
        }
    }

    pub fn get_mixer(&self) -> Arc<Mutex<Mixer>> {
        Arc::clone(&self.mixer)
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn state(&self) -> AudioEngineState {
        *self.state.lock().unwrap()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

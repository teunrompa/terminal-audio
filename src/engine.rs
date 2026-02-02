use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use cpal::{
    Device, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossterm::event::{KeyCode, KeyEvent};

use crate::mixer::Mixer;

pub struct AudioEngine {
    sample_rate: f32,
    state: Arc<Mutex<AudioEngineState>>,
    stream_config: SupportedStreamConfig,
    channels: usize,
    device: Device,
    mixer: Arc<Mutex<Mixer>>,
}

#[derive(PartialEq, Clone)]
pub enum AudioEngineState {
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
            state: Arc::new(Mutex::new(AudioEngineState::Stopping)),
            device,
            stream_config: config,
            channels,
            mixer: Arc::new(Mutex::new(Mixer::new(sample_rate))),
        })
    }

    pub fn get_engine_state(&'_ self) -> MutexGuard<'_, AudioEngineState> {
        self.state.lock().unwrap()
    }

    pub fn get_mixer(&'_ self) -> MutexGuard<'_, Mixer> {
        self.mixer.lock().unwrap()
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        self.mixer.lock().unwrap().handle_keyboard_input(key_event);

        match key_event.code {
            KeyCode::Char('p') => self.handle_stop_start(),
            KeyCode::Char('w') => todo!(), //TODO: implement next window
            _ => {}
        }
    }

    pub fn handle_stop_start(&mut self) {
        let mut current_state = self.state.lock().unwrap();

        *current_state = match *current_state {
            AudioEngineState::Playing => AudioEngineState::Stopping,
            AudioEngineState::Stopping => AudioEngineState::Playing,
        }
    }

    pub fn process(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let channels = self.channels;
        let device = self.device.clone();
        let stream_config = self.stream_config.clone();

        let mixer = Arc::clone(&self.mixer);
        let state = Arc::clone(&self.state);

        let _handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send>> {
            let stream = device
                .build_output_stream(
                    &stream_config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        for frame in data.chunks_mut(channels) {
                            let mixer_output = mixer.lock().unwrap().get_output();
                            //TODO: This should be conunously running
                            // Write to all channels
                            for sample in frame.iter_mut() {
                                *sample = mixer_output; // Reduced volume
                            }
                        }
                    },
                    |err| eprintln!("Output error: {}", err),
                    None,
                )
                .expect("error building output stream");

            stream.play().expect("error could not play stream");

            loop {
                {
                    let current_state = state.lock().unwrap();

                    if *current_state == AudioEngineState::Stopping {
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(100)); //Restearting loop evry 100ms
            }

            Ok(())
        });

        Ok(())
    }
}

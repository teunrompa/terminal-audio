use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rodio::{Source, StreamError, mixer::Mixer, source::SineWave};

//Main playback and processing point
pub struct AudioEngine {
    state: Arc<Mutex<AudioEngineState>>,
    audio_thread: Option<thread::JoinHandle<()>>,
}

enum AudioEngineState {
    Playing,
    Stopping,
}

impl AudioEngine {
    pub fn new() -> Self {
        AudioEngine {
            state: Arc::new(Mutex::new(AudioEngineState::Playing)),
            audio_thread: None,
        }
    }

    pub fn run(&mut self) -> Result<(), StreamError> {
        let state = Arc::clone(&self.state);

        let handle = thread::spawn(move || {
            if let Ok(steam_handle) = rodio::OutputStreamBuilder::open_default_stream() {
                let duration = Duration::from_secs_f32(3.0);
                let mixer = steam_handle.mixer();
                let source = SineWave::new(74.0).amplify(0.3).take_duration(duration);
                mixer.add(source);

                if let Ok(mut s) = state.lock() {
                    *s = AudioEngineState::Playing;
                }

                thread::sleep(duration);

                if let Ok(mut s) = state.lock() {
                    *s = AudioEngineState::Stopping;
                }
            }
        });

        self.audio_thread = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            *state = AudioEngineState::Stopping;
        }

        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(AudioEngineState::Playing)),
            audio_thread: None,
        }
    }
}

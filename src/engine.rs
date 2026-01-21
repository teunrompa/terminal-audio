use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rand::Rng;
use ratatui::widgets::{Bar, BarChart, BarGroup, Widget};
use rodio::{Source, StreamError, source::SineWave};

use crate::track::Track;

//Main playback and processing point
pub struct AudioEngine {
    state: Arc<Mutex<AudioEngineState>>,
    audio_thread: Option<thread::JoinHandle<()>>,
    tracks: Vec<Track>,
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
            tracks: Vec::new(),
        }
    }

    //TODO: this implementation is currently only for testing
    pub fn new_track(&mut self) {
        let mut rng = rand::rng();
        self.tracks.push(Track::new(
            rng.random_range(1.0..10.0),
            Some(Box::new(SineWave::new(44.0))),
            "New Track".to_string(),
        ));
    }

    pub fn get_tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    pub fn run(&mut self) -> Result<(), StreamError> {
        let state = Arc::clone(&self.state);

        //Spawn seprate thread for audio processing
        let handle = thread::spawn(move || {
            if let Ok(steam_handle) = rodio::OutputStreamBuilder::open_default_stream() {
                //TODO: This needs to be implemented for Track
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
        AudioEngine::new()
    }
}

impl Widget for &AudioEngine {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let current_tracks = self.get_tracks();

        let mut bars = vec![];

        for track in current_tracks {
            let bar = Bar::with_label(
                track.get_name(),
                track.get_volume() as u64, //TODO: Can cause problems in the fututre due to float conversions
            );

            bars.push(bar);
        }

        let group = BarGroup::new(bars);

        BarChart::default()
            .bar_width(5)
            .bar_gap(5)
            .data(group)
            .render(area, buf);
    }
}

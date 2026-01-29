use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use crate::track::Track;

//Sums all levels from track and mixes it together
//Also used for managing track behavoir on a high level
pub struct Mixer {
    sample_rate: f64,
    tracks: HashMap<usize, Track>,
    selected_track: usize,
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            tracks: HashMap::new(),
            sample_rate: 0.0,
            selected_track: 0,
        }
    }

    pub fn prepare(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    //Create new track and store the id in the hashmap
    pub fn add_track(&mut self, volume: f32, name: String) {
        let last_track_id = self.tracks.len();
        self.tracks.insert(last_track_id, Track::new(volume, name));

        self.selected_track = last_track_id;
    }

    pub fn remove_track_at(&mut self, id: usize) {
        self.tracks.remove_entry(&id);
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('t') => self.add_track(10.0, "new track".to_string()),
            KeyCode::Char('r') => self.remove_track_at(self.selected_track),
            _ => {}
        }
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

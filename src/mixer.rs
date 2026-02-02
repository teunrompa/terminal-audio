use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Style,
    widgets::{Bar, BarChart, BarGroup, Widget},
};

use crate::track::Track;

//Sums all levels from track and mixes it together
//Also used for managing track behavoir on a high level
pub struct Mixer {
    sample_rate: f32,
    tracks: HashMap<usize, Track>,
    selected_track: usize,
    master_volume: f32,
    increment_volume: f32,
}

impl Mixer {
    pub fn new(sample_rate: f32) -> Self {
        Mixer {
            tracks: HashMap::new(),
            selected_track: 0,
            master_volume: 0.3,
            sample_rate,
            increment_volume: 0.1,
        }
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn get_output(&mut self) -> f32 {
        let mut output = 0.0;

        for track in self.tracks.iter_mut() {
            output += track.1.get_output();
        }

        output *= self.master_volume;

        //Limit the volume
        output.clamp(-1.0, 1.0)
    }

    //Create new track and store the id in the hashmap
    pub fn add_track(&mut self, volume: f32, name: String) {
        let last_track_id = self.tracks.len();
        self.tracks
            .insert(last_track_id, Track::new(volume, name, self.sample_rate));
    }

    fn next_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if self.selected_track < self.tracks.len() - 1 {
            self.selected_track += 1
        } else {
            self.selected_track = 0;
        }
    }

    fn previous_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if self.selected_track > 0 {
            self.selected_track -= 1
        } else {
            self.selected_track = self.tracks.len() - 1; //Loop around
        }
    }

    fn remove_track_at(&mut self, id: usize) {
        self.tracks.remove_entry(&id);
    }

    fn get_selected_track(&mut self) -> Option<&mut Track> {
        self.tracks.get_mut(&self.selected_track)
    }

    fn increment_selected_track_volume(&mut self) {
        let increment = self.increment_volume;
        let track = self.get_selected_track();

        if let Some(track) = track {
            track.increse_volume(increment);
        }
    }

    fn decrease_selected_track_volume(&mut self) {
        let increment = self.increment_volume;
        let track = self.get_selected_track();

        if let Some(track) = track {
            track.decrease_volume(increment);
        }
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('t') => self.add_track(0.3, "new track".to_string()),
            KeyCode::Char('r') => self.remove_track_at(self.selected_track),
            KeyCode::Right => self.next_track(),
            KeyCode::Left => self.previous_track(),
            KeyCode::Up => self.increment_selected_track_volume(),
            KeyCode::Down => self.decrease_selected_track_volume(),
            _ => {}
        }
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl Widget for &Mixer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut bars = vec![];

        //Determine bar color
        let mut track_ids: Vec<_> = self.tracks.keys().collect();
        track_ids.sort();

        //Sorts rendering order for the tracks
        for id in track_ids {
            let track = &self.tracks[id];
            let track_name = format!("{} new track", id);

            let style = if *id == self.selected_track {
                Style::default().bold().red()
            } else {
                Style::default().blue()
            };

            bars.push(
                Bar::default()
                    .value((track.get_volume() * 100.0) as u64)
                    .label("Volume")
                    .text_value(format!("{track_name:>}"))
                    .style(style),
            );
        }

        let group = BarGroup::new(bars);

        BarChart::default()
            .bar_gap(10)
            .bar_width(7)
            .data(group)
            .max(100)
            .render(area, buf);
    }
}

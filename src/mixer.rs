use std::collections::HashMap;

use crossterm::{
    event::{KeyCode, KeyEvent},
    style::Color,
};
use ratatui::{
    style::Style,
    widgets::{Bar, BarChart, BarGroup, Widget},
};

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
    }

    pub fn next_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if self.selected_track < self.tracks.len() - 1 {
            self.selected_track += 1
        } else {
            self.selected_track = 0;
        }
    }
    pub fn previous_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if self.selected_track > 0 {
            self.selected_track -= 1
        } else {
            self.selected_track = self.tracks.len() - 1; //Loop around
        }
    }

    pub fn remove_track_at(&mut self, id: usize) {
        self.tracks.remove_entry(&id);
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('t') => self.add_track(10.0, "new track".to_string()),
            KeyCode::Char('r') => self.remove_track_at(self.selected_track),
            KeyCode::Right => self.next_track(),
            KeyCode::Left => self.previous_track(),
            _ => {}
        }
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
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
                    .style(style)
                    .value(track.get_volume() as u64)
                    .text_value(track_name),
            );
        }

        let group = BarGroup::new(bars);

        BarChart::default()
            .bar_gap(10)
            .bar_width(10)
            .data(group)
            .render(area, buf);
    }
}

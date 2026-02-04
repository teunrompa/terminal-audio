use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Widget},
};

use crate::{
    generators::{Envelope, PrimitiveWave, WaveType},
    track::Track,
};

//Sums all levels from track and mixes it together
//Also used for managing track behavoir on a high level
pub struct Mixer {
    sample_rate: f32,
    tracks: HashMap<usize, Track>,
    track_order: Vec<usize>,
    selected_index: usize,
    master_volume: f32,
    increment_volume: f32,
    bpm: f32,
    next_id: usize,
}

impl Mixer {
    pub fn new(sample_rate: f32, bpm: f32) -> Self {
        Mixer {
            tracks: HashMap::new(),
            track_order: Vec::new(),
            selected_index: 0,
            master_volume: 1.0,
            sample_rate,
            increment_volume: 0.1,
            bpm,
            next_id: 0,
        }
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    //Main audio processing function
    pub fn process_block(&mut self, num_samples: usize) -> Vec<f32> {
        let mut mix = vec![0.0f32; num_samples];

        self.tracks.iter_mut().for_each(|(_i, track)| {
            let track_output = track.process_block(num_samples);

            for (i, sample) in track_output.iter().enumerate() {
                mix[i] += sample;
            }
        });

        for sample in &mut mix {
            *sample = (*sample * self.master_volume).tanh(); //Softclipping 
            *sample = sample.clamp(-1.0, 1.0);
        }

        mix
    }

    //Create new track and store the id in the hashmap
    pub fn add_track(
        &mut self,
        volume: f32,
        name: String,
        length: usize,
        step_division: u8,
        sample_rate: f32,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let mut track = Track::new(volume, name, sample_rate, self.bpm, length, step_division);

        track.set_instrument(Box::new(PrimitiveWave::new(
            144.0,
            WaveType::Sine,
            sample_rate,
            Envelope::new(0.010, 0.01, 1.0, 0.03, sample_rate),
        )));

        self.tracks.insert(id, track);

        self.track_order.push(id);

        self.selected_index = self.track_order.len() - 1;
    }

    fn next_track(&mut self) {
        if self.track_order.is_empty() {
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.track_order.len();
    }

    fn previous_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        self.selected_index = if self.selected_index == 0 {
            self.track_order.len() - 1
        } else {
            self.selected_index - 1
        };
    }

    pub fn remove_selected_track(&mut self) {
        if self.track_order.is_empty() {
            return;
        }

        let id = self.track_order[self.selected_index];
        self.tracks.remove(&id);
        self.track_order.remove(self.selected_index);

        if self.selected_index >= self.track_order.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn selected_track(&mut self) -> Option<&mut Track> {
        self.track_order
            .get(self.selected_index)
            .and_then(|id| self.tracks.get_mut(id))
    }

    pub fn get_track_id(&mut self, id: usize) -> Option<&mut Track> {
        self.tracks.get_mut(&id)
    }

    //TODO: Set max volume
    fn increment_selected_track_volume(&mut self) {
        let inccrement = self.increment_volume;
        if let Some(track) = self.selected_track() {
            track.increse_volume(inccrement);
        }
    }

    fn decrease_selected_track_volume(&mut self) {
        let increment = self.increment_volume;
        if let Some(track) = self.selected_track() {
            track.decrease_volume(increment);
        }
    }

    pub fn set_master_volumne(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 2.0);
    }

    //Updates bpm for all tracks
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;

        for track in self.tracks.values_mut() {
            track.set_bpm(bpm);
        }
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;

        for track in self.tracks.values_mut() {
            track.set_sample_rate(sample_rate);
        }
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            //TODO: implement params from the UI
            KeyCode::Char('t') => self.add_track(
                0.3,
                format!("Track {}", self.next_id),
                16,
                16,
                self.sample_rate,
            ),
            KeyCode::Char('r') => self.remove_selected_track(),
            KeyCode::Right => self.next_track(),
            KeyCode::Left => self.previous_track(),
            KeyCode::Up => self.increment_selected_track_volume(),
            KeyCode::Down => self.decrease_selected_track_volume(),
            _ => {}
        }
    }
}

impl Widget for &Mixer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.track_order.is_empty() {
            return;
        }

        let bars: Vec<Bar> = self
            .track_order
            .iter()
            .enumerate()
            .map(|(idx, id)| {
                let track = &self.tracks[id];
                let is_selected = idx == self.selected_index;

                let vol_percent = (track.get_volume() * 100.0) as u64;

                let style = if is_selected {
                    Style::default().fg(Color::Cyan).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Gray)
                };

                Bar::default()
                    .value(vol_percent.clamp(0, 100))
                    .label(track.get_name())
                    .text_value(format!("Vol% {}", vol_percent))
                    .style(style)
            })
            .collect();

        BarChart::default()
            .block(
                Block::default()
                    .title(format!(
                        "Mixer | BPM {:.1} | Master: {:.0}%",
                        self.bpm,
                        self.master_volume * 100.0
                    ))
                    .borders(Borders::ALL),
            )
            .bar_width(10)
            .bar_gap(2)
            .data(BarGroup::new(bars))
            .max(100)
            .render(area, buf);
    }
}

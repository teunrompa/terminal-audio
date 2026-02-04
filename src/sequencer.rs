use std::vec;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

//The sequencer knows where all the events are in the sequnce
pub struct Sequencer {
    events: Vec<Option<NoteEvent>>,
    bpm: f32,
    sample_rate: f32,
    current_step: usize,
    samples_per_step: f32,
    samples_accumulated: f32,
    step_division: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct NoteEvent {
    pub frequency: f32,
    pub velocity: f32,
}

impl Sequencer {
    pub fn new(bpm: f32, sample_rate: f32, length: usize, step_division: u8) -> Self {
        // Calculate samples per step
        let beats_per_second = bpm / 60.0;
        let samples_per_beat = sample_rate / beats_per_second;
        let samples_per_step = samples_per_beat / step_division as f32;

        Sequencer {
            events: vec![None; length],
            bpm,
            sample_rate,
            current_step: 0,
            samples_accumulated: 0.0,
            samples_per_step,
            step_division,
        }
    }

    //Check step boundry. Returns true when boundry is hit
    pub fn process(&mut self, num_samples: usize) -> bool {
        self.samples_accumulated += num_samples as f32;

        if self.samples_accumulated >= self.samples_per_step {
            self.samples_accumulated -= self.samples_per_step;
            self.advance_step();
            true
        } else {
            false
        }
    }

    //goto next step loop around when limit is reached
    fn advance_step(&mut self) {
        self.current_step = (self.current_step + 1) % self.events.len();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        let beats_per_second = self.bpm / 60.0;
        let samples_per_beat = sample_rate / beats_per_second;
        self.samples_per_step = samples_per_beat / self.step_division as f32;
    }

    //Gets the current events if there are any
    pub fn get_current_event(&self) -> Option<&NoteEvent> {
        self.events.get(self.current_step).and_then(|e| e.as_ref())
    }

    pub fn set_note_at(&mut self, step: usize, frequency: f32, velocity: f32) {
        if let Some(slot) = self.events.get_mut(step) {
            *slot = Some(NoteEvent {
                frequency,
                velocity,
            });
        }
    }

    pub fn clear_step(&mut self, step: usize) {
        if let Some(slot) = self.events.get_mut(step) {
            *slot = None;
        }
    }

    pub fn get_event(&self, id: usize) -> &Option<NoteEvent> {
        if let Some(event) = self.events.get(id) {
            event
        } else {
            &None
        }
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.samples_accumulated = 0.0;
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        let beats_per_second = bpm / 60.0;
        let samples_per_beat = self.sample_rate / beats_per_second;
        self.samples_per_step = samples_per_beat / self.step_division as f32;
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn pattern_len(&self) -> usize {
        self.events.len()
    }
}

impl Widget for &Sequencer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // Calculate step width
        let step_width = area.width / self.events.len() as u16;

        for (step_idx, event) in self.events.iter().enumerate() {
            let x = area.x + (step_idx as u16 * step_width);
            let cell_area = Rect {
                x,
                y: area.y,
                width: step_width.saturating_sub(1), // Leave 1 char spacing
                height: area.height,
            };

            // Determine style based on state
            let style = if step_idx == self.current_step {
                // Current playing step - bright highlight
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else if event.is_some() {
                // Step has a note - filled
                Style::default().bg(Color::Blue)
            } else {
                // Empty step
                Style::default().bg(Color::DarkGray)
            };

            // Render the step cell
            let block = Block::default().style(style);
            block.render(cell_area, buf);

            // Optionally show note info
            if let Some(note) = event {
                let freq_text = format!("{:.0}Hz", note.frequency);
                buf.set_string(x, area.y, &freq_text, Style::default().fg(Color::White));
            }
        }
    }
}

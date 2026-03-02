use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Widget};

use crate::generators::Instrument;
use crate::sequencer::Sequencer;

//Contains state of the voulume and the sound source, processes all items on the chain
//endpoint of sound goes to mixer
pub struct Track {
    sample_rate: f32,
    volume: f32,
    name: String,
    sequencer: Sequencer,
    bpm: f32,
    instrument: Option<Box<dyn Instrument + Send>>,
}

impl Track {
    pub fn new(
        volume: f32,
        name: String,
        sample_rate: f32,
        bpm: f32,
        length: usize,
        step_division: u8,
    ) -> Self {
        Track {
            sample_rate,
            volume,
            name,
            instrument: None,
            sequencer: Sequencer::new(bpm, sample_rate, length, step_division),
            bpm,
        }
    }

    pub fn set_instrument(&mut self, instrument: Box<dyn Instrument>) {
        self.instrument = Some(instrument);
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    //Processes a buffer of audio
    pub fn process_block(&mut self, num_samples: usize) -> Vec<f32> {
        let mut output = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            if let Some(instrument) = self.instrument.as_mut() {
                if self.sequencer.process(1) {
                    if let Some(note) = self.sequencer.get_current_event() {
                        instrument.note_on(note.frequency);
                    } else if instrument.get_envelope().is_active() {
                        //If no note is found trigger the release state
                        instrument.note_off(); //Note off triggers release state
                    }
                }

                let sample = instrument.process(); //Process also moves the phase 
                output.push(sample * self.volume);
            } else {
                output.push(0.0);
            }
        }

        output
    }

    pub fn sequencer_mut(&mut self) -> &mut Sequencer {
        &mut self.sequencer
    }

    pub fn decrease_volume(&mut self, amount: f32) {
        self.volume -= amount;

        if self.volume <= 0.0 {
            self.volume = 0.0;
        }
    }

    pub fn increse_volume(&mut self, amount: f32) {
        self.volume += amount;
    }

    pub fn sequencer(&self) -> &Sequencer {
        &self.sequencer
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sequencer.set_sample_rate(sample_rate)
    }

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}

//TODO: implement colums for the sequencer in a tracker style way
impl Widget for &Track {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vol_percent = (self.volume * 100.0) as u16;

        //Device container with title
        let block = Block::default()
            .title(self.name.as_str())
            .borders(Borders::ALL);

        let inner = block.inner(area);
        block.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // instrument name
                Constraint::Min(1),     // volume gauge
                Constraint::Length(60), //Padding
            ])
            .split(inner);

        // Instrument name
        let instrument_name = self
            .instrument
            .as_ref()
            .map(|i| i.get_name().to_string())
            .unwrap_or_else(|| "No Instrument".to_string());

        //Instrument label
        Paragraph::new(instrument_name)
            .style(Style::default().fg(Color::Gray))
            .render(layout[0], buf);

        // Volume gauge
        Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(vol_percent.clamp(0, 100))
            .label(format!("Vol {}%", vol_percent))
            .render(layout[1], buf);
    }
}

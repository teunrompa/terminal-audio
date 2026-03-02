use std::{collections::HashMap, fmt::format};

use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub trait Instrument: Send {
    fn get_name(&self) -> &str;
    fn process(&mut self) -> f32;
    fn note_on(&mut self, frequency: f32);
    fn note_off(&mut self);
    fn get_envelope(&self) -> &Envelope;
    fn get_phase(&self) -> f32;
}

pub trait Processor {
    type Input;
    type Output;

    fn process(&mut self, input: Self::Input) -> Self::Output;
}

pub enum WaveType {
    Sine,
    Square,
    Triangle,
    Saw,
}

//TODO: we should be abale to select any instrument type put it in a list and select the
//appropriate instrument
pub struct PrimitiveWave {
    wave_type: WaveType,
    sample_rate: f32,
    phase: f32,
    frequency: f32,
    envelope: Envelope,
}

pub enum DeviceType {
    Fx,
    Synth,
    Analyzer,
}

pub struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32, //level in db
    release: f32,
    phase: f32,
    state: EnvelopeState,
    current_level: f32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

impl Envelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) -> Self {
        Envelope {
            attack: attack * sample_rate,
            decay: decay * sample_rate,
            sustain,
            release: release * sample_rate,
            phase: 0.0,
            state: EnvelopeState::Idle,
            current_level: 0.0,
        }
    }

    pub fn start(&mut self) {
        self.state = EnvelopeState::Attack;
        self.phase = 0.0;
    }

    pub fn is_idle(&self) -> bool {
        self.state == EnvelopeState::Idle
    }

    pub fn stop(&mut self) {
        self.state = EnvelopeState::Release;
        if self.state != EnvelopeState::Release && self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.phase = 0.0;
        }
    }

    pub fn get_state(&self) -> EnvelopeState {
        self.state
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            EnvelopeState::Attack | EnvelopeState::Decay | EnvelopeState::Sustain
        )
    }
}

impl Processor for Envelope {
    type Input = f32;

    type Output = f32;

    fn process(&mut self, input: Self::Input) -> Self::Output {
        let amp = match self.state {
            EnvelopeState::Attack => {
                let t = (self.phase / self.attack).min(1.0); // Clamp to 1.0
                let amp = self.current_level + (1.0 - self.current_level) * t; //Smooth

                self.phase += 1.0;
                if self.phase >= self.attack {
                    self.state = EnvelopeState::Decay;
                    self.phase = 0.0;
                }
                amp
            }
            EnvelopeState::Decay => {
                let t = (self.phase / self.decay).min(1.0); // Normalized 0-1
                let amp = 1.0 - ((1.0 - self.sustain) * t);
                self.phase += 1.0;
                if self.phase >= self.decay {
                    self.state = EnvelopeState::Sustain;
                    self.phase = 0.0;
                }
                amp
            }
            EnvelopeState::Sustain => self.sustain,
            EnvelopeState::Release => {
                let t = (self.phase / self.release).min(1.0); // Normalized 0-1
                let amp = self.sustain * (1.0 - t);
                self.phase += 1.0;
                if self.phase >= self.release {
                    self.state = EnvelopeState::Idle;
                }

                amp
            }
            EnvelopeState::Idle => {
                self.phase = 0.0;
                0.0
            }
        };

        self.current_level = amp;
        input * amp
    }
}

impl PrimitiveWave {
    pub fn new(frequency: f32, wave_type: WaveType, sample_rate: f32, envelope: Envelope) -> Self {
        PrimitiveWave {
            wave_type,
            sample_rate,
            phase: 0.0,
            frequency,
            envelope,
        }
    }

    pub fn advance_phase(&mut self) {
        self.phase += self.frequency / self.sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0
        }
    }

    pub fn get_ui(&self, frame: &mut Frame) {
        let block = Block::new()
            .border_style(Style::new().blue())
            .title("Parameters");

        frame.render_widget(block, frame.area());
    }

    fn get_envelope_mut(&mut self) -> &mut Envelope {
        &mut self.envelope
    }
}

impl Instrument for PrimitiveWave {
    fn process(&mut self) -> f32 {
        let wave_result = match self.wave_type {
            WaveType::Sine => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            WaveType::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveType::Triangle => {
                if self.phase < 0.5 {
                    // Rising: 0 -> 0.5 becomes -1 -> 1
                    4.0 * self.phase - 1.0
                } else {
                    // Falling: 0.5 -> 1 becomes 1 -> -1
                    -4.0 * self.phase + 3.0
                }
            }
            WaveType::Saw => {
                // Linear ramp from -1 to 1
                2.0 * self.phase - 1.0
            }
        };

        self.advance_phase();

        self.envelope.process(wave_result)
    }

    fn note_on(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.envelope.start();
    }

    fn note_off(&mut self) {
        self.envelope.stop();
    }

    fn get_envelope(&self) -> &Envelope {
        &self.envelope
    }

    fn get_phase(&self) -> f32 {
        self.phase
    }

    fn get_name(&self) -> &str {
        "Primitive Wave"
    }
}

impl Widget for &Envelope {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        //Expose the adsr value
        let params = [
            ("attack", self.attack),
            ("decay", self.decay),
            ("sustain", self.sustain),
            ("release", self.release),
        ];

        //Draw the parameters
        let lines: Vec<Line> = params
            .iter()
            .map(|(name, value)| {
                Line::from(vec![
                    Span::raw(format!("{:<12}", name)),
                    Span::raw(format!("{:.3}", value)),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .title_top("Primitive Wave Synth")
                .borders(Borders::ALL),
        );

        paragraph.render(area, buf);
    }
}

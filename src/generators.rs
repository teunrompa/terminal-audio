use crate::engine::AudioEngineState;

pub trait Instrument: Send {
    fn get_current_sample(&self) -> f32;
}

pub trait Processor {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Self::Output;
}

pub enum Oscilator {
    Sine,
    Square,
    Triangle,
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
    sample_rate: f32,
    state: EnvelopeState,
}

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
            attack,
            decay,
            sustain,
            release,
            phase: 0.0,
            sample_rate,
            state: EnvelopeState::Idle,
        }
    }

    pub fn process(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Attack => {
                let amp = self.phase / self.attack;
                self.phase += 1.0;
                if self.phase >= self.attack {
                    self.state = EnvelopeState::Decay;
                    self.phase = 0.0;
                }
                amp
            }
            EnvelopeState::Decay => {
                let amp = 1.0 - ((1.0 - self.sustain) * (self.phase / self.decay));
                self.phase += 1.0;
                if self.phase >= self.decay {
                    self.state = EnvelopeState::Sustain;
                }
                amp
            }
            EnvelopeState::Sustain => self.sustain,
            EnvelopeState::Release => {
                let amp = self.sustain * (1.0 - (self.phase / self.release));
                self.phase += 1.0;
                if self.phase >= self.release {
                    self.state = EnvelopeState::Idle;
                }

                amp
            }
            EnvelopeState::Idle => 0.0,
        }
    }
}

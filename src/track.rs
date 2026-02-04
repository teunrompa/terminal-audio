use crate::generators::{EnvelopeState, Instrument};
use crate::sequencer::Sequencer;

//Contains state of the voulume and the sound source, processes all items on the chain
//endpoint of sound goes to mixer
pub struct Track {
    sample_rate: f32,
    phase: f32,
    volume: f32,
    name: String,
    sequencer: Sequencer,
    bpm: f32,
    note_phase: f32,
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
            phase: 0.0,
            volume,
            name,
            instrument: None,
            sequencer: Sequencer::new(bpm, sample_rate, length, step_division),
            note_phase: 0.0,
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

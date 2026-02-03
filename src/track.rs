use crate::generators::Instrument;
use crate::sequencer::{NoteEvent, Sequencer};

//Contains state of the voulume and the sound source, processes all items on the chain
//endpoint of sound goes to mixer
pub struct Track {
    sample_rate: f32,
    phase: f32,
    volume: f32,
    name: String,
    sequencer: Sequencer,
    current_note: Option<NoteEvent>,
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
            current_note: None,
            note_phase: 0.0,
            bpm,
        }
    }

    pub fn get_current_note(&self) -> Option<NoteEvent> {
        self.current_note
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
            if self.sequencer.process(1) {
                if let Some(note) = self.sequencer.get_current_event() {
                    self.trigger_note(*note);
                } else {
                    self.current_note = None;
                }
            }

            let sample = self.genrate_sample();
            output.push(sample * self.volume);
        }

        output
    }

    //Processes a single sample
    pub fn get_output(&mut self) -> f32 {
        if self.sequencer.process(1) {
            if let Some(note) = self.sequencer.get_current_event() {
                self.trigger_note(*note);
            } else {
                self.current_note = None
            }
        }

        self.genrate_sample() * self.volume
    }

    pub fn sequencer_mut(&mut self) -> &mut Sequencer {
        &mut self.sequencer
    }

    pub fn trigger_note(&mut self, note: NoteEvent) {
        self.current_note = Some(note);
        self.note_phase = 0.0;
    }

    pub fn genrate_sample(&mut self) -> f32 {
        match &self.current_note {
            Some(note) => {
                let sample = (self.note_phase * 2.0 * std::f32::consts::PI).sin();

                self.note_phase += note.frequency / self.sample_rate;

                if self.note_phase > 1.0 {
                    self.note_phase = -1.0;
                }

                sample * note.velocity
            }
            None => 0.0,
        }
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

    //For testing purposes only
    pub fn play_sine_440hz(&mut self) -> f32 {
        let current_sample = (self.phase * 2.0 * std::f32::consts::PI).sin();

        self.phase += 440.0 / self.sample_rate;

        current_sample
    }

    //For testing purposes only
    pub fn play_sine(&mut self, frequency: f32) -> f32 {
        let current_sample = (self.phase * 2.0 * std::f32::consts::PI).sin();

        self.phase += frequency / self.sample_rate;

        current_sample
    }

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}

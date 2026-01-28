pub struct Pattern {
    steps: Vec<Step>,
    length: usize,
    current_step: usize,
}

#[derive(Default, Clone)]
pub struct Step {
    pub active: bool,
    pub velocity: u8,
    pub note: Option<u8>,
    pub probability: f32,
}

pub struct Clock {
    bpm: f32,
    sample_rate: u32,
    samples_per_step: f32,
    sample_counter: f32,
    ppqn: u32, //pulses per quarter note
}

pub struct Sequencer {
    patterns: Vec<Pattern>,
    active_pattern: usize,
    clock: Clock,
    playing: bool,
}

impl Sequencer {
    pub fn new(bpm: f32, sample_rate: u32) -> Self {
        Sequencer {
            patterns: vec![Pattern::new(16)],
            active_pattern: 0,
            clock: Clock::new(bpm, sample_rate, 16),
            playing: false,
        }
    }

    pub fn process(&mut self) -> Option<Step> {
        if !self.playing {
            return None;
        }

        if self.clock.tick() {
            self.patterns[self.active_pattern].advance().cloned()
        } else {
            None
        }
    }
}

impl Pattern {
    pub fn new(num_steps: usize) -> Self {
        Pattern {
            steps: vec![Step::default(); num_steps],
            length: num_steps,
            current_step: 0,
        }
    }

    pub fn advance(&mut self) -> Option<&Step> {
        let step = &self.steps[self.current_step];
        self.current_step = (self.current_step + 1) % self.length;

        if step.active { Some(step) } else { None }
    }
}

impl Clock {
    pub fn new(bpm: f32, sample_rate: u32, steps_per_beat: u32) -> Self {
        let samples_per_step = (60.0 / bpm) * sample_rate as f32 / steps_per_beat as f32;

        Clock {
            bpm,
            sample_rate,
            samples_per_step,
            sample_counter: 0.0,
            ppqn: 24,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.sample_counter += 1.0;

        if self.sample_counter >= self.samples_per_step {
            self.sample_counter -= self.samples_per_step;
            true
        } else {
            false
        }
    }

    pub fn set_bpm(&mut self, bpm: f32, steps_per_beat: u32) {
        self.bpm = bpm;

        self.samples_per_step = (60.0 / bpm) * self.sample_rate as f32 / steps_per_beat as f32;
    }
}

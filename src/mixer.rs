use crate::track::Track;

pub struct Mixer {
    sample_rate: f64,
    tracks: Vec<Track>,
}

impl Mixer {
    pub fn new() -> Self {
        Mixer {
            tracks: Vec::new(),
            sample_rate: 0.0,
        }
    }

    pub fn prepare(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }
}

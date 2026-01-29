//Contains state of the voulume and the sound source
pub struct Track {
    sample_rate: f32,
    phase: f32,
    volume: f32,
    name: String,
    instrument: Option<Box<dyn Instrument + Send>>,
}

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

impl Track {
    pub fn new(volume: f32, name: String, sample_rate: f32) -> Self {
        Track {
            sample_rate,
            phase: 0.0,
            volume,
            name,
            instrument: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_output(&mut self) -> f32 {
        let sample = self.play_sine_440hz();
        sample * self.get_volume()
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn decrease_volume(&mut self, amount: f32) {
        self.volume -= amount;
    }

    pub fn increse_volume(&mut self, amount: f32) {
        self.volume += amount;
    }

    //For testing purposes only
    pub fn play_sine_440hz(&mut self) -> f32 {
        let current_sample = (self.phase * 2.0 * std::f32::consts::PI).sin();

        self.phase += 440.0 / self.sample_rate;

        current_sample
    }

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}

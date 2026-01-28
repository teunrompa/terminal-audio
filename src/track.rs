//Contains state of the voulume and the sound source
pub struct Track {
    volume: f32,
    name: String,
    devices: Vec<Device>,
}

pub struct Device {
    device_type: DeviceType,
}

pub enum DeviceType {
    Fx,
    Synth,
    Analyzer,
}

impl Track {
    pub fn new(volume: f32, name: String) -> Self {
        Track {
            volume,
            name,
            devices: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
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

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}

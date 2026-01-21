use rodio::Source;

//Contains state of the voulume and the sound source
pub struct Track {
    volume: f32,
    source: Option<Box<dyn Source>>, //Source option for a track witch can also be empty
    name: String,
}

impl Track {
    pub fn new(volume: f32, source: Option<Box<dyn Source>>, name: String) -> Self {
        Track {
            volume,
            source,
            name,
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

    pub fn change_source(&mut self, new_source: Option<Box<dyn Source>>) {
        self.source = new_source;
    }
}

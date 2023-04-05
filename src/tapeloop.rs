use nih_plug::nih_dbg;

pub struct TAPESTATE {
    samplerate: f32,
    length: f32,
    speed: f32,
    buffer: Vec<f32>,
    pub current_sample_idx: usize,
}

impl Default for TAPESTATE {
    fn default() -> Self {
        Self {
            samplerate: 44100.0,
            length: 2.0,
            speed: 1.0,
            buffer: vec![0.0; 44100],
            current_sample_idx: 0,
        }
    }
}

impl TAPESTATE {
    pub fn init(&mut self, samplerate: f32) {
        self.samplerate = samplerate;
        self.current_sample_idx = 0;
        self.buffer = vec![0.0; (self.samplerate * self.length) as usize];
    }

    pub fn inc_sample_idx(&mut self) {
        if (self.current_sample_idx == self.buffer.len() - 1) {
            self.current_sample_idx = 0;
        } else {
            self.current_sample_idx += 1;
        }
    }

    pub fn to_buffer(&mut self, sample: &mut f32, gain: Option<f32>) {
        self.buffer[self.current_sample_idx] += sample.clone() * gain.unwrap_or(1.0);
    }

    pub fn from_buffer(&mut self) -> f32 {
        return self.buffer[self.current_sample_idx];
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

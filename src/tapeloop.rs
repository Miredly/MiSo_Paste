use nih_plug::nih_dbg;

const MAX_TAPE_LENGTH: f32 = 60.0;
const MAX_TAPE_SPEED: f32 = 2.0;

#[derive(Clone)]
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
            length: 6.0,
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
        self.buffer = vec![0.0; (self.samplerate * MAX_TAPE_LENGTH) as usize];
    }

    pub fn inc_sample_idx(&mut self) {
        if self.current_sample_idx >= self.end_of_loop() {
            self.current_sample_idx = 0;
        } else {
            self.current_sample_idx += 1;
        }
    }

    pub fn dec_sample_idx(&mut self) {
        if self.current_sample_idx == 0 {
            self.current_sample_idx = self.end_of_loop();
        } else {
            self.current_sample_idx -= 1;
        }
    }

    pub fn fast_forward(&mut self) {
        for _ in 0..MAX_TAPE_SPEED as i32 {
            self.inc_sample_idx();
        }
    }

    pub fn to_buffer(&mut self, sample: &mut f32, gain: Option<f32>) {
        self.buffer[self.current_sample_idx] += sample.clone() * gain.unwrap_or(1.0);
    }

    pub fn from_buffer(&mut self) -> f32 {
        return self.buffer[self.current_sample_idx];
    }

    pub fn set_tape_length(&mut self, len: f32) {
        let current_len = self.length;
        let new_len = f32::clamp(len, 1.0, MAX_TAPE_LENGTH);

        self.length = new_len;
    }

    pub fn set_tape_speed(&mut self, speed: f32) {
        self.speed = f32::clamp(speed, 0.1, MAX_TAPE_SPEED);
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
    }

    pub fn current_position_percent(&mut self) -> f32 {
        self.current_sample_idx as f32 / self.end_of_loop() as f32
    }

    fn end_of_loop(&mut self) -> usize {
        return f32::clamp(
            self.length * self.samplerate,
            self.samplerate,
            (self.buffer.len() - 1) as f32,
        ) as usize;
    }
}

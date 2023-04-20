use nih_plug::nih_dbg;

const MAX_TAPE_LENGTH: f32 = 60.0;
const MAX_TAPE_SPEED: f32 = 3.0;

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
        self.buffer = vec![0.0; (self.samplerate * MAX_TAPE_LENGTH) as usize];
    }

    pub fn inc_sample_idx(&mut self) {
        if (self.current_sample_idx >= self.end_of_loop()) {
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

    pub fn set_tape_length(&mut self, len: f32) {
        let current_len = self.length;
        let new_len = f32::clamp(len, 1.0, MAX_TAPE_LENGTH);

        self.length = new_len;
    }

    pub fn set_tape_speed(&mut self, speed: f32) {
        self.speed = f32::clamp(speed, 0.1, MAX_TAPE_SPEED);
    }

    pub fn clear(&mut self) {
        nih_dbg!("clear called");
        //TODO - come up with a more elegant solution to make sure this only happens once per click
        if self.buffer[0] != 0.0 {
            self.buffer.fill(0.0);
        } else {
            self.buffer[0] += 0.1; //quick & dirty way to make sure the panic button will always at least work on the second click
        }
    }

    fn end_of_loop(&mut self) -> usize {
        //CURSED I KNOW
        return f32::clamp(
            self.length * self.samplerate,
            self.samplerate,
            (self.buffer.len() - 1) as f32,
        ) as usize;
    }
}

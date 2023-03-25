//translated from the C++ example here: https://www.musicdsp.org/en/latest/Filters/236-3-band-equaliser.html

use std::f32::consts::PI;

const VSA: f32 = f32::EPSILON;
const LOWFREQ: f32 = 880.0;
const HIGHFREQ: f32 = 5000.0;

#[derive(Clone, Copy)]
pub struct EQSTATE {
    //filter 1
    pub lf: f32,
    pub f1p0: f32,
    pub f1p1: f32,
    pub f1p2: f32,
    pub f1p3: f32,

    //filter2
    pub hf: f32,
    pub f2p0: f32,
    pub f2p1: f32,
    pub f2p2: f32,
    pub f2p3: f32,

    //sample history
    pub sdm1: f32,
    pub sdm2: f32,
    pub sdm3: f32,

    //gain controls
    pub lg: f32,
    pub mg: f32,
    pub hg: f32,

    //samplerate
    pub sr: f32,
}

impl EQSTATE {
    pub fn init(&mut self, samplerate: f32) {
        self.sr = samplerate;

        self.lg = 1.0;
        self.mg = 1.0;
        self.hg = 1.0;

        self.set_highband_frequency(HIGHFREQ);
        self.set_lowband_frequency(LOWFREQ);
    }

    pub fn set_lowband_frequency(&mut self, frequency: f32) {
        self.lf = self.calculate_bandpass_frequency(frequency);
    }

    pub fn set_highband_frequency(&mut self, frequency: f32) {
        self.hf = self.calculate_bandpass_frequency(frequency)
    }

    fn calculate_bandpass_frequency(&self, frequency: f32) -> f32 {
        return 2.0 * f32::sin(PI * (frequency / self.sr));
    }

    pub fn process_3band<'a>(&'a mut self, sample: &'a mut f32) -> &'a mut f32 {
        let mut l: f32;
        let mut m: f32;
        let mut h: f32;

        //lowpass
        self.f1p0 += (self.lf * (*sample - self.f1p0)) + VSA;
        self.f1p1 += self.lf * (self.f1p0 - self.f1p1);
        self.f1p2 += self.lf * (self.f1p1 - self.f1p2);
        self.f1p3 += self.lf * (self.f1p2 - self.f1p3);

        l = self.f1p3;

        //highpass
        self.f2p0 += (self.hf * (*sample - self.f2p0)) + VSA;
        self.f2p1 += self.hf * (self.f2p0 - self.f2p1);
        self.f2p2 += self.hf * (self.f2p1 - self.f2p2);
        self.f2p3 += self.hf * (self.f2p2 - self.f2p3);

        h = self.sdm3 - self.f2p3;

        //calc midrange (signal - (low + high))
        m = self.sdm3 - (h + l);

        //scale combine and store
        l *= self.lg;
        m *= self.mg;
        h *= self.hg;

        //shuffle history buffer
        self.sdm3 = self.sdm2;
        self.sdm2 = self.sdm1;
        self.sdm1 = sample.clone();

        *sample = l + m + h;

        return sample;
    }
}

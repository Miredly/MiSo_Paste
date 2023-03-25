use nih_plug::prelude::*;
use std::{borrow::BorrowMut, f32::consts::PI, sync::Arc};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started
#[derive(Clone, Copy)]
struct EQSTATE {
    //filter 1
    lf: f32,
    f1p0: f32,
    f1p1: f32,
    f1p2: f32,
    f1p3: f32,

    //filter2
    hf: f32,
    f2p0: f32,
    f2p1: f32,
    f2p2: f32,
    f2p3: f32,

    //sample history
    sdm1: f32,
    sdm2: f32,
    sdm3: f32,

    //gain controls
    lg: f32,
    mg: f32,
    hg: f32,

    //samplerate
    sr: f32,
}

impl EQSTATE {
    fn SetLowpassFrequency(&mut self, frequency: f32) {
        self.lf = self.CalculateBandpassFrequency(frequency);
    }

    fn CalculateBandpassFrequency(&self, frequency: f32) -> f32 {
        return 2.0 * f32::sin(PI * (frequency / self.sr));
    }
}

const LOWFREQ: f32 = 880.0;
const HIGHFREQ: f32 = 5000.0;

const VSA: f32 = f32::EPSILON;
struct MisoFirst {
    params: Arc<MisoFirstParams>,
    es: EQSTATE,
}

impl Default for MisoFirst {
    fn default() -> Self {
        Self {
            params: Arc::new(MisoFirstParams::default()),
            es: EQSTATE {
                lf: 0.0,
                f1p0: 0.0,
                f1p1: 0.0,
                f1p2: 0.0,
                f1p3: 0.0,

                hf: 0.0,
                f2p0: 0.0,
                f2p1: 0.0,
                f2p2: 0.0,
                f2p3: 0.0,

                sdm1: 0.0,
                sdm2: 0.0,
                sdm3: 0.0,

                lg: 1.0,
                mg: 1.0,
                hg: 1.0,

                sr: 44100.0,
            },
        }
    }
}

#[derive(Params)]
struct MisoFirstParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "low gain"]
    pub low_gain: FloatParam,
    #[id = "mid gain"]
    pub mid_gain: FloatParam,
    #[id = "high gain"]
    pub high_gain: FloatParam,
}

impl Default for MisoFirstParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            low_gain: FloatParam::new(
                "low gain",
                1.0,
                FloatRange::Linear {
                    min: 0.01,
                    max: 2.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            mid_gain: FloatParam::new(
                "mid gain",
                1.0,
                FloatRange::Linear {
                    min: 0.01,
                    max: 2.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            high_gain: FloatParam::new(
                "high gain",
                1.0,
                FloatRange::Linear {
                    min: 0.01,
                    max: 2.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
        }
    }
}

impl Plugin for MisoFirst {
    const NAME: &'static str = "Miso First";
    const VENDOR: &'static str = "Miredly";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "miles@mired.space";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        //init EQ STATE
        self.es.sr = _buffer_config.sample_rate;
        self.es.lg = 1.0;
        self.es.mg = 1.0;
        self.es.hg = 1.0;

        // self.es.lf = 2.0 * f32::sin(PI * (LOWFREQ / _buffer_config.sample_rate));
        self.es.hf = 2.0 * f32::sin(PI * (HIGHFREQ / _buffer_config.sample_rate));
        self.es.SetLowpassFrequency(LOWFREQ);

        nih_warn!("Finished init");
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();

            self.es.lg = self.params.low_gain.smoothed.next();
            self.es.mg = self.params.mid_gain.smoothed.next();
            self.es.hg = self.params.high_gain.smoothed.next();

            for sample in channel_samples {
                do_3band(sample, self.es.borrow_mut());
                *sample *= gain;
            }
        }

        return ProcessStatus::Normal;
    }
}

fn do_3band<'a>(sample: &'a mut f32, es: &mut EQSTATE) -> &'a mut f32 {
    let mut l: f32;
    let mut m: f32;
    let mut h: f32;

    //lowpass
    es.f1p0 += (es.lf * (sample.clone() - es.f1p0)) + VSA;
    es.f1p1 += es.lf * (es.f1p0 - es.f1p1);
    es.f1p2 += es.lf * (es.f1p1 - es.f1p2);
    es.f1p3 += es.lf * (es.f1p2 - es.f1p3);

    l = es.f1p3;

    //highpass
    es.f2p0 += (es.hf * (sample.clone() - es.f2p0)) + VSA;
    es.f2p1 += es.hf * (es.f2p0 - es.f2p1);
    es.f2p2 += es.hf * (es.f2p1 - es.f2p2);
    es.f2p3 += es.hf * (es.f2p2 - es.f2p3);

    h = es.sdm3 - es.f2p3;

    //calc midrange (signal - (low + high))
    m = es.sdm3 - (h + l);

    //scale combine and store
    l *= es.lg;
    m *= es.mg;
    h *= es.hg;

    //shuffle history buffer
    es.sdm3 = es.sdm2;
    es.sdm2 = es.sdm1;
    es.sdm1 = sample.clone();

    *sample = l + m + h;

    return sample;
}

impl ClapPlugin for MisoFirst {
    const CLAP_ID: &'static str = "com.miso.miso-first";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("first stab at a pluggo, hey");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for MisoFirst {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(MisoFirst);
nih_export_vst3!(MisoFirst);
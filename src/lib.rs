mod eq;
pub use crate::eq::EQSTATE;
use nih_plug::prelude::*;
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct MisoFirst {
    params: Arc<MisoFirstParams>,
    es: EQSTATE,
}

impl Default for MisoFirst {
    fn default() -> Self {
        Self {
            params: Arc::new(MisoFirstParams::default()),
            es: EQSTATE::default(),
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
    #[id = "low frequency"]
    pub low_frequency: FloatParam,
    #[id = "high frequency"]
    pub high_frequency: FloatParam,
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

            low_frequency: FloatParam::new(
                "low frequency",
                880.0,
                FloatRange::Linear {
                    min: 110.0,
                    max: 5000.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            high_frequency: FloatParam::new(
                "high frequency",
                5000.0,
                FloatRange::Linear {
                    min: 5000.0,
                    max: 12000.0,
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
        self.es.init(_buffer_config.sample_rate);

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
            //get input
            let gain = self.params.gain.smoothed.next();

            self.es.lg = self.params.low_gain.smoothed.next();
            self.es.mg = self.params.mid_gain.smoothed.next();
            self.es.hg = self.params.high_gain.smoothed.next();

            self.es
                .set_lowband_frequency(self.params.low_frequency.smoothed.next());
            self.es
                .set_highband_frequency(self.params.high_frequency.smoothed.next());

            //processing
            for sample in channel_samples {
                self.es.process_3band(sample);
                *sample *= gain;
            }
        }

        return ProcessStatus::Normal;
    }
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

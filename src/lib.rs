mod eq;
pub use crate::eq::EQSTATE;
mod im;
pub use crate::im::UiImages;
mod tapeloop;
pub use crate::tapeloop::TAPESTATE;
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, Color32, Id, LayerId, Order, Rounding},
    widgets, EguiState,
};
use std::sync::Arc;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 150.0;

const SLIDER_Y_POS: f32 = 45.0;
const SLIDER_HORIZONTAL_SPACING: f32 = 45.0;
const BUTTON_WIDTH: f32 = 50.0;
const BUTTON_HEIGHT: f32 = 25.0;

struct MisoFirst {
    params: Arc<MisoFirstParams>,
    es: EQSTATE,
    tape: TAPESTATE,
    //GUI stuff
    peak_meter_decay_weight: f32,
    peak_meter: Arc<AtomicF32>,
    tape_pos: Arc<AtomicF32>,
    images: UiImages,
}

impl Default for MisoFirst {
    fn default() -> Self {
        Self {
            params: Arc::new(MisoFirstParams::default()),
            es: EQSTATE::default(),
            tape: TAPESTATE::default(),
            //GUI
            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            tape_pos: Arc::new(AtomicF32::new(0.0)),
            images: UiImages::default(),
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
    #[id = "tape speed"]
    pub tape_speed: FloatParam,
    #[id = "tape length"]
    pub tape_length: FloatParam,
    #[id = "clear"]
    pub clear: BoolParam,
    #[id = "erase"]
    pub erase: BoolParam,

    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,
}

impl Default for MisoFirstParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                0.01,
                FloatRange::Linear {
                    min: 0.01,
                    max: 1.0,
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

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

            tape_speed: FloatParam::new(
                "tape speed",
                1.0,
                FloatRange::Linear {
                    min: 0.10,
                    max: 3.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            tape_length: FloatParam::new(
                "tape length",
                2.0,
                FloatRange::Linear {
                    min: 2.0,
                    max: 60.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            clear: BoolParam::new("clear", false),

            erase: BoolParam::new("erase", false),

            editor_state: EguiState::from_size(512, 512),
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

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        fn rect_from_point(x: f32, y: f32) -> egui::Rect {
            egui::Rect::from_two_pos(egui::pos2(x, y), egui::pos2(x + 1.0, y + 1.0))
        }
        fn button_rect(xpos: f32, ypos: f32) -> egui::Rect {
            egui::Rect::from_center_size(
                egui::pos2(xpos, ypos),
                egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT),
            )
        }

        let params = self.params.clone();
        let peak_meter = self.peak_meter.clone();
        let images = self.images.clone();
        let tape_pos = self.tape_pos.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // NOTE: See `plugins/diopser/src/editor.rs` for an example using the generic UI widget
                    ui.spacing_mut().slider_width = 300.0;
                    //IMAGES
                    let background_texture = ui.ctx().load_texture(
                        "background",
                        images.background.to_owned(),
                        egui::TextureFilter::Linear,
                    );

                    let background_image =
                        egui::Image::new(&background_texture, egui::vec2(512.0, 512.0));
                    ui.put(
                        egui::Rect::from_points(&[egui::pos2(0.0, 0.0), egui::pos2(512.0, 512.0)]),
                        background_image,
                    );

                    let reel_texture = ui.ctx().load_texture(
                        "reel",
                        images.reel.to_owned(),
                        egui::TextureFilter::Linear,
                    );

                    let reel_image = egui::Image::new(&reel_texture, egui::vec2(128.0, 128.0))
                        .rotate(
                            tape_pos.load(std::sync::atomic::Ordering::Relaxed)
                                * 360.0_f32.to_radians(),
                            egui::vec2(0.5, 0.5),
                        );
                    ui.put(
                        egui::Rect::from_center_size(
                            egui::pos2(256.0, 256.0),
                            egui::vec2(128.0, 128.0),
                        ),
                        reel_image,
                    );

                    ui.put(
                        egui::Rect::from_center_size(
                            egui::pos2(400.0, 256.0),
                            egui::vec2(128.0, 128.0),
                        ),
                        reel_image,
                    );

                    //SLIDERS
                    //gain
                    let gain_slider =
                        egui::widgets::Slider::from_get_set(
                            0.0..=1.0,
                            |new_value| match new_value {
                                Some(value) => {
                                    let new_value = value as f32;

                                    setter.begin_set_parameter(&params.gain);
                                    setter.set_parameter(&params.gain, new_value);
                                    setter.end_set_parameter(&params.gain);

                                    value
                                }
                                None => params.gain.value() as f64,
                            },
                        )
                        .vertical();
                    ui.put(rect_from_point(10.0, SLIDER_Y_POS), gain_slider);

                    //tape length
                    let tape_length_slider = egui::widgets::Slider::from_get_set(
                        0.25..=60.0,
                        |new_value| match new_value {
                            Some(value) => {
                                let new_value = value as f32;

                                setter.begin_set_parameter(&params.tape_length);
                                setter.set_parameter(&params.tape_length, new_value);
                                setter.end_set_parameter(&params.tape_length);

                                value
                            }
                            None => params.tape_length.value() as f64,
                        },
                    )
                    .vertical();

                    ui.put(
                        rect_from_point(10.0 + SLIDER_HORIZONTAL_SPACING, SLIDER_Y_POS),
                        tape_length_slider,
                    );

                    //BUTTONS
                    //panic button
                    let panic_button = egui::Button::new("Panic");
                    setter.begin_set_parameter(&params.clear);
                    if ui.put(button_rect(475.0, 350.0), panic_button).clicked() {
                        setter.set_parameter(&params.clear, true);
                    } else {
                        setter.set_parameter(&params.clear, false);
                    }
                    setter.end_set_parameter(&params.clear);

                    //PEAK METER
                    // TODO: Add a proper custom widget instead of reusing a progress bar
                    let peak_meter =
                        util::gain_to_db(peak_meter.load(std::sync::atomic::Ordering::Relaxed));
                    let peak_meter_text = if peak_meter > util::MINUS_INFINITY_DB {
                        format!("{peak_meter:.1} dBFS")
                    } else {
                        String::from("-inf dBFS")
                    };

                    let peak_meter_normalized = (peak_meter + 60.0) / 60.0;
                    ui.allocate_space(egui::Vec2::splat(2.0));

                    let peak_meter_widget = egui::widgets::ProgressBar::new(peak_meter_normalized)
                        .text(peak_meter_text);

                    ui.put(
                        egui::Rect {
                            min: egui::pos2(10.0, 460.0),
                            max: egui::pos2(500.0, 480.0),
                        },
                        peak_meter_widget,
                    );
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        //init EQ STATE
        self.es.init(buffer_config.sample_rate);
        //init TAPESTATE
        self.tape.init(buffer_config.sample_rate);

        // After `PEAK_METER_DECAY_MS` milliseconds of pure silence, the peak meter's value should
        // have dropped by 12 dB
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

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
        if self.params.clear.value() {
            nih_dbg!("clear");
            self.tape.clear();
        }

        for channel_samples in buffer.iter_samples() {
            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();

            //get input
            let gain = self.params.gain.smoothed.next();

            self.es.lg = self.params.low_gain.smoothed.next();
            self.es.mg = self.params.mid_gain.smoothed.next();
            self.es.hg = self.params.high_gain.smoothed.next();

            self.tape
                .set_tape_length(self.params.tape_length.smoothed.next());
            self.tape
                .set_tape_speed(self.params.tape_speed.smoothed.next());

            //processing
            for sample in channel_samples {
                //EQ
                self.es.process_3band(sample);
                //TAPE
                self.tape.inc_sample_idx();
                if (self.params.erase.value()) {
                    self.tape.to_buffer(&mut 0.0, Some(gain));
                } else {
                    self.tape.to_buffer(sample, Some(gain));
                }
                *sample += self.tape.from_buffer();
                amplitude += *sample;
            }

            if self.params.editor_state.is_open() {
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed);

                self.tape_pos.store(
                    self.tape.current_position_percent(),
                    std::sync::atomic::Ordering::Relaxed,
                );
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

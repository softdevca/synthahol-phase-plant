use std::io::{Error, Read, Seek};

use music_note::midi;
use uom::si::f32::{Frequency, Ratio, Time};
use uom::si::frequency::hertz;
use uom::si::ratio::percent;
use uom::si::time::{millisecond, second};

use crate::effect::{Distortion, Filter};
use crate::generator::*;
use crate::point::{CurvePoint, CurvePointMode};
use crate::*;

// TODO: Make GeneratorBlock crate-private.

/// All generators are stored in the preset using the same structure. Generators
/// are converted to and from this structure for reading and writing. Having
/// specific generators makes the models more clear than having everything in
/// one giant block.
#[derive(Clone, Debug)]
pub struct GeneratorBlock {
    pub id: GeneratorId,
    pub mode: GeneratorMode,
    pub enabled: bool,
    pub minimized: bool,
    pub name: String,

    // Stored in the string pool
    pub settings_locked: bool,

    #[doc(alias = "semi_cent")]
    pub fine_tuning: f32,

    pub harmonic: f32,
    pub shift: Frequency,
    pub rate: Rate,

    /// Percentage of 360 degrees.
    pub phase_offset: Ratio,

    /// Percentage of 360 degrees.
    pub phase_jitter: Ratio,

    pub level: Ratio,
    pub unison: Unison,

    // Analog generator
    pub analog_waveform: AnalogWaveform,
    pub sync_multiplier: f32,
    pub pulse_width: Ratio,

    /// Where the sample starts playing
    pub offset_position: Ratio,
    pub offset_locked: bool,

    /// Beginning of the loop area
    pub loop_start_position: Ratio,
    pub loop_locked: bool,
    pub loop_length: Ratio,
    pub loop_enabled: bool,

    pub crossfade_amount: Ratio,
    pub invert: bool,
    pub filter_effect: Filter,
    pub distortion_effect: Distortion,
    pub band_limit: Frequency,

    /// Mix/aux level
    pub mix_level: Ratio,

    // Noise generator
    pub noise_waveform: NoiseWaveform,
    pub noise_slope: Decibels,
    pub stereo: Ratio,
    pub seed_mode: SeedMode,
    pub pan: Ratio,

    pub output_enabled: bool,
    pub output_gain: Decibels,
    pub output_destination: OutputDestination,

    pub envelope: Envelope,

    pub wavetable_contents: Vec<u8>,
    pub wavetable_edited: bool,
    pub wavetable_frame: f32,
    pub wavetable_name: Option<String>,
    pub wavetable_path: Option<String>,

    /// The Y range is -1.0..=1.0
    pub curve: Vec<CurvePoint>,

    pub curve_edited: bool,
    pub curve_length: Time,
    pub curve_name: Option<String>,
    pub curve_path: Option<String>,
    pub curve_loop_mode: LoopMode,
    pub curve_loop_start: Ratio,
    pub curve_loop_length: Ratio,

    /// A file containing the samples to play back. Usually in a format like FLAC, MP3 or WAV.
    /// If there are no sample contents then it is a factory sample.
    pub sample_contents: Vec<u8>,

    pub sample_rate: f32,
    pub sample_name: Option<String>,
    pub sample_path: Option<String>,
    pub sample_loop_mode: LoopMode,

    #[doc(alias = "root note")]
    pub base_pitch: f32,
    pub base_pitch_locked: bool,

    pub granular_position: Ratio,
    pub granular_direction: GranularDirection,
    pub granular_envelope: GranularEnvelope,
    pub granular_align_phases: bool,
    pub granular_grains: f32,
    pub granular_grain_length: Time,
    pub granular_auto_grain_length: bool,
    pub granular_spawn_rate_mode: GranularSpawnRateMode,
    pub granular_randomization: GranularRandomization,
    pub granular_chord: GranularChord,
    pub granular_warm_start: bool,
}

impl GeneratorBlock {
    /// Size on disk in bytes.
    pub(crate) const SIZE: usize = 200;

    pub(crate) fn read_data_block<R: Read + Seek>(
        &mut self,
        reader: &mut PhasePlantReader<R>,
    ) -> Result<(), Error> {
        use GeneratorMode::*;
        match self.mode {
            CurveOutput => {
                // Curve data block that contains the shape.
                let point_count = reader.read_u32()?;
                self.curve = Vec::with_capacity(point_count as usize);
                for _ in 0..point_count {
                    self.curve.push(CurvePoint {
                        x: reader.read_f32()?,
                        y: reader.read_f32()?,
                        curve_x: reader.read_f32()?,
                        curve_y: reader.read_f32()?,
                        mode: CurvePointMode::from_id(reader.read_u32()?)?,
                    });
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// The defaults match what is stored in the blank areas of the init preset.
impl Default for GeneratorBlock {
    fn default() -> Self {
        let mut distortion_effect = Distortion::new();
        distortion_effect.drive = Decibels::from_linear(1.0);
        distortion_effect.dynamics = Ratio::zero();

        Self {
            id: 0,
            mode: GeneratorMode::Blank,
            name: "".to_owned(),
            enabled: true,
            minimized: false,
            settings_locked: false,
            fine_tuning: 0.0,
            harmonic: 1.0,
            shift: Frequency::zero(),
            rate: Rate {
                sync: false,
                frequency: Frequency::new::<hertz>(100.0),
                numerator: 4,
                denominator: NoteValue::Sixteenth,
            },
            phase_offset: Ratio::zero(),
            phase_jitter: Ratio::zero(),
            level: Ratio::new::<percent>(100.0),
            unison: Default::default(),
            analog_waveform: AnalogWaveform::Saw,
            sync_multiplier: 1.0,
            pulse_width: Ratio::new::<percent>(50.0),
            offset_position: Ratio::zero(),
            offset_locked: false,
            loop_start_position: Ratio::zero(),
            loop_locked: false,
            loop_length: Ratio::zero(),
            loop_enabled: false,
            crossfade_amount: Ratio::zero(),
            invert: false,
            distortion_effect,
            filter_effect: Filter {
                cutoff: Frequency::new::<hertz>(440.0), // Different than default Filter effect
                gain: Decibels::ZERO,
                ..Default::default()
            },
            mix_level: Ratio::new::<percent>(100.0),
            noise_waveform: NoiseWaveform::Colored,
            noise_slope: Decibels::new(3.0103),
            stereo: Ratio::zero(),
            seed_mode: Default::default(),
            pan: Ratio::zero(),

            output_enabled: true,
            output_gain: Decibels::from_linear(0.25),
            output_destination: OutputDestination::Lane1,

            envelope: Default::default(),
            band_limit: Frequency::new::<hertz>(22050.0),

            wavetable_contents: Vec::new(),
            wavetable_edited: false,
            wavetable_frame: 0.0,
            wavetable_name: None,
            wavetable_path: None,

            curve: Vec::new(),
            curve_edited: false,
            curve_length: Time::new::<second>(1.0),
            curve_name: None,
            curve_path: None,
            curve_loop_mode: LoopMode::Off,
            curve_loop_start: Ratio::zero(),
            curve_loop_length: Ratio::new::<percent>(100.0),

            sample_contents: Vec::new(),
            sample_rate: 0.0,
            sample_name: None,
            sample_path: None,
            sample_loop_mode: LoopMode::Infinite,

            base_pitch: midi!(C, 4).into_byte() as f32,
            base_pitch_locked: false,

            // Granulator generator
            granular_position: Ratio::zero(),
            granular_direction: GranularDirection::Start,
            granular_envelope: Default::default(),
            granular_align_phases: false,
            granular_grains: 1.0,
            granular_grain_length: Time::new::<millisecond>(250.0),
            granular_auto_grain_length: true,
            granular_spawn_rate_mode: GranularSpawnRateMode::Density,
            granular_randomization: Default::default(),
            granular_chord: Default::default(),
            granular_warm_start: false,
        }
    }
}

//
// Conversion of generators to and from blocks
//

impl From<&AnalogOscillator> for GeneratorBlock {
    fn from(gen: &AnalogOscillator) -> Self {
        Self {
            id: gen.id,
            name: gen.name(),
            mode: gen.mode(),
            enabled: gen.enabled,
            fine_tuning: gen.tuning,
            harmonic: gen.harmonic,
            shift: gen.shift,
            phase_offset: gen.phase_offset,
            phase_jitter: gen.phase_jitter,
            level: gen.level,
            pulse_width: gen.pulse_width,
            sync_multiplier: gen.sync_multiplier,
            unison: gen.unison,
            analog_waveform: gen.waveform,
            ..Default::default()
        }
    }
}

impl From<&AuxRouting> for GeneratorBlock {
    fn from(gen: &AuxRouting) -> Self {
        Self {
            id: gen.id,
            name: gen.name(),
            mode: gen.mode(),
            enabled: gen.enabled,
            invert: gen.invert,
            mix_level: gen.level,
            ..Default::default()
        }
    }
}

impl From<&BlankGenerator> for GeneratorBlock {
    fn from(generator: &BlankGenerator) -> Self {
        Self {
            mode: generator.mode(),
            name: generator.name(),

            // The blocks for blank generators have slightly different defaults
            // than other generators. For example, the block for the Group
            // generator has an output destination of Lane 1 even though it
            // has no output.
            envelope: Envelope {
                decay: Time::new::<second>(0.001),
                ..Default::default()
            },
            output_destination: OutputDestination::None,
            unison: Unison {
                voices: 1,
                ..Default::default()
            },

            ..Default::default()
        }
    }
}

impl From<&DistortionEffect> for GeneratorBlock {
    fn from(generator: &DistortionEffect) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            distortion_effect: generator.effect.clone(),
            ..Default::default()
        }
    }
}

impl From<&CurveOutput> for GeneratorBlock {
    fn from(generator: &CurveOutput) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            settings_locked: generator.settings_locked,
            output_enabled: generator.output_enabled,
            output_gain: generator.gain,
            pan: generator.pan,
            output_destination: generator.destination,
            curve: generator.curve.clone(),
            curve_edited: generator.curve_edited,
            curve_length: generator.curve_length,
            curve_name: generator.curve_name.clone(),
            curve_path: generator.curve_path.clone(),
            curve_loop_mode: generator.loop_mode,
            curve_loop_start: generator.loop_start,
            curve_loop_length: generator.loop_start,
            ..Default::default()
        }
    }
}

impl From<&EnvelopeOutput> for GeneratorBlock {
    fn from(generator: &EnvelopeOutput) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            output_enabled: generator.output_enabled,
            output_gain: generator.gain,
            pan: generator.pan,
            output_destination: generator.destination,
            envelope: generator.envelope.clone(),
            ..Default::default()
        }
    }
}

impl From<&FilterEffect> for GeneratorBlock {
    fn from(generator: &FilterEffect) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            filter_effect: generator.effect.clone(),
            ..Default::default()
        }
    }
}

impl From<&GranularGenerator> for GeneratorBlock {
    fn from(gen: &GranularGenerator) -> Self {
        Self {
            id: gen.id,
            name: gen.name(),
            mode: gen.mode(),
            enabled: gen.enabled,
            fine_tuning: gen.fine_tuning,
            harmonic: gen.harmonic,
            shift: gen.shift,
            phase_offset: gen.phase_offset,
            phase_jitter: gen.phase_jitter,
            level: gen.level,

            sample_contents: gen.sample_contents.clone(),
            sample_name: gen.sample_name.clone(),
            sample_path: gen.sample_path.clone(),

            base_pitch: gen.base_pitch,
            base_pitch_locked: gen.base_pitch_locked,

            granular_position: gen.position,
            granular_direction: gen.direction,
            granular_envelope: gen.envelope.clone(),
            granular_align_phases: gen.align_phases,
            granular_grains: gen.grains,
            granular_grain_length: gen.grain_length,
            granular_auto_grain_length: gen.auto_grain_length,
            granular_spawn_rate_mode: gen.spawn_rate_mode,
            granular_randomization: gen.randomization.clone(),
            granular_chord: gen.chord.clone(),
            granular_warm_start: gen.warm_start,

            ..Default::default()
        }
    }
}

impl From<&Group> for GeneratorBlock {
    fn from(generator: &Group) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            minimized: generator.minimized,
            ..Default::default()
        }
    }
}

impl From<&MixRouting> for GeneratorBlock {
    fn from(generator: &MixRouting) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            mix_level: generator.level,
            invert: generator.invert,
            ..Default::default()
        }
    }
}

impl From<&NoiseGenerator> for GeneratorBlock {
    fn from(generator: &NoiseGenerator) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            fine_tuning: generator.semi_cent,
            harmonic: generator.harmonic,
            shift: generator.shift,
            phase_offset: generator.phase_offset,
            phase_jitter: generator.phase_jitter,
            level: generator.level,
            noise_waveform: generator.waveform,
            noise_slope: generator.slope,
            stereo: generator.stereo,
            seed_mode: generator.seed_mode,
            ..Default::default()
        }
    }
}

impl From<&SamplePlayer> for GeneratorBlock {
    fn from(generator: &SamplePlayer) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            fine_tuning: generator.semi_cent,
            harmonic: generator.harmonic,
            shift: generator.shift,
            phase_offset: generator.phase_offset,
            phase_jitter: generator.phase_jitter,
            level: generator.level,
            unison: generator.unison,
            offset_locked: generator.offset_locked,
            offset_position: generator.offset_position,
            loop_locked: generator.loop_locked,
            loop_start_position: generator.loop_start_position,
            loop_length: generator.loop_length,
            loop_enabled: generator.loop_enabled,
            sample_loop_mode: generator.loop_mode,
            crossfade_amount: generator.crossfade_amount,
            sample_contents: generator.sample_contents.clone(),
            sample_name: generator.sample_name.clone(),
            sample_path: generator.sample_path.clone(),
            base_pitch: generator.base_pitch,
            base_pitch_locked: generator.base_pitch_locked,
            ..Default::default()
        }
    }
}

impl From<&WavetableOscillator> for GeneratorBlock {
    fn from(generator: &WavetableOscillator) -> Self {
        Self {
            id: generator.id,
            mode: generator.mode(),
            name: generator.name(),
            enabled: generator.enabled,
            fine_tuning: generator.tuning,
            harmonic: generator.harmonic,
            shift: generator.shift,
            phase_offset: generator.phase_offset,
            phase_jitter: generator.phase_jitter,
            level: generator.level,
            wavetable_frame: generator.frame,
            band_limit: generator.band_limit,
            unison: generator.unison,
            wavetable_name: generator.wavetable_name.clone(),
            wavetable_path: generator.wavetable_path.clone(),
            ..Default::default()
        }
    }
}

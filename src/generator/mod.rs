//! Generators create and affect audio.

use std::any::Any;

use downcast_rs::{Downcast, impl_downcast};
use strum_macros::Display;

pub use analog_oscillator::*;
pub use aux_routing::*;
pub use blank::*;
pub use curve_output::*;
pub use distortion_effect::*;
pub use envelope_output::*;
pub use filter_effect::*;
pub use granular_generator::*;
pub use group::*;
pub use mix_routing::*;
pub use noise_generator::*;
pub use sample_player::*;
pub use wavetable_oscillator::*;

use crate::*;

mod analog_oscillator;
mod aux_routing;
mod blank;
mod curve_output;
mod distortion_effect;
mod envelope_output;
mod filter_effect;
mod granular_generator;
mod group;
mod mix_routing;
mod noise_generator;
mod sample_player;
mod wavetable_oscillator;

pub type GeneratorId = u16;

/// The sample player does not include the `Off` option.
#[derive(Copy, Clone, Debug, Display, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum LoopMode {
    // The discriminants correspond to the file format.
    Off = 0,
    Infinite = 1,
    Sustain = 2,
    PingPong = 3,
    Reverse = 4,
}

impl LoopMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown loop mode {id}")))
    }
}

/// Not the same discriminants as [`LaneDestination`].
#[derive(Copy, Clone, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum OutputDestination {
    None = 0,
    Lane1 = 1,
    Lane2 = 2,
    Lane3 = 3,
    Master = 4,
    Sideband = 5,
}

impl OutputDestination {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown output destination {id}"),
            )
        })
    }
}

impl Display for OutputDestination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use OutputDestination::*;
        let label = match self {
            None => "None",
            Lane1 => "Lane 1",
            Lane2 => "Lane 2",
            Lane3 => "Lane 3",
            Master => "Master",
            Sideband => "Sideband",
        };
        f.write_str(label)
    }
}

#[derive(Copy, Clone, Debug, Default, Display, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum SeedMode {
    // The discriminates match the file format. Using an enumeration provides
    // a clearer intention than a boolean.
    #[default]
    Stable,
    Random,
}

impl SeedMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown seed mode {id}")))
    }
}

impl Preset {
    /// Get a generator by index. Useful for testing.
    pub fn generator<T: Generator>(&self, generator_index: usize) -> Option<&T> {
        self.generators.get(generator_index)?.downcast_ref::<T>()
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Display, Eq, FromRepr, PartialEq)]
pub enum GeneratorMode {
    // The discriminants correspond to the file format.
    AnalogOscillator = 2,
    AuxRouting = 8,
    Blank = 0,
    CurveOutput = 11,
    DistortionEffect = 6,
    EnvelopeOutput = 10,
    FilterEffect = 7,
    GranularGenerator = 12,
    Group = 1,
    MixRouting = 9,
    NoiseGenerator = 3,
    #[doc(alias = "Sampler")]
    SamplePlayer = 4,
    WavetableOscillator = 5,
}

impl GeneratorMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown generator mode {id}"),
            )
        })
    }

    pub fn is_blank(&self) -> bool {
        self == &GeneratorMode::Blank
    }

    pub fn name(&self) -> &'static str {
        use GeneratorMode::*;
        match self {
            AnalogOscillator => "Analog",
            AuxRouting => "Aux",
            Blank => "Blank",
            CurveOutput => "Curve",
            DistortionEffect => "Distortion",
            EnvelopeOutput => "Envelope",
            FilterEffect => "Filter",
            GranularGenerator => "Granular",
            Group => "Group",
            MixRouting => "Mix",
            NoiseGenerator => "Noise",
            SamplePlayer => "Sampler",
            WavetableOscillator => "Wavetable",
        }
    }
}

pub trait Generator: Downcast + std::fmt::Debug {
    /// Not every generator has an assignable ID. The blank generator in
    /// particular does not.
    fn id(&self) -> Option<GeneratorId>;

    fn as_block(&self) -> GeneratorBlock;
    fn box_eq(&self, other: &dyn Any) -> bool;
    fn is_enabled(&self) -> bool;
    fn mode(&self) -> GeneratorMode;
    fn name(&self) -> String;

    /// Not all generators have presets
    fn set_preset_name(&mut self, _preset_name_opt: Option<String>) {}
}

impl_downcast!(Generator);

impl PartialEq for Box<dyn Generator> {
    fn eq(&self, other: &Box<dyn Generator>) -> bool {
        self.box_eq(other.as_any())
    }
}

#[cfg(test)]
mod test {
    use crate::effect::{DistortionMode, FilterMode};
    use crate::test::read_preset;

    use super::*;

    /// Preset with every generator in one group. At least one property of each generator is tested.
    #[test]
    fn all_version_1() {
        let preset = read_preset("generators", "generators-all-1.8.13.phaseplant");
        let generator_names = preset
            .generators
            .iter()
            .map(|gen| gen.name())
            .collect::<Vec<String>>();
        assert_eq!(
            generator_names,
            [
                "Group",
                "Analog",
                "Noise",
                "Sampler",
                "Wavetable",
                "Distortion",
                "Filter",
                "Aux",
                "Mix",
                "Envelope",
            ]
        );

        let group: &Group = preset.generator(0).unwrap();
        assert!(!group.minimized);

        let analog: &AnalogOscillator = preset.generator(1).unwrap();
        assert_eq!(analog.waveform, AnalogWaveform::Saw);

        let noise: &NoiseGenerator = preset.generator(2).unwrap();
        assert_eq!(noise.seed_mode, SeedMode::Stable);

        let sampler: &SamplePlayer = preset.generator(3).unwrap();
        assert!(!sampler.loop_enabled);

        let wavetable: &WavetableOscillator = preset.generator(4).unwrap();
        assert_eq!(wavetable.phase_jitter, Ratio::zero());

        let distortion: &DistortionEffect = preset.generator(5).unwrap();
        assert_eq!(distortion.effect.mode, DistortionMode::Overdrive);

        let filter: &FilterEffect = preset.generator(6).unwrap();
        assert_eq!(filter.effect.filter_mode, FilterMode::LowPass);

        let aux: &AuxRouting = preset.generator(7).unwrap();
        assert!(!aux.invert);

        let mix: &MixRouting = preset.generator(8).unwrap();
        assert_eq!(mix.level.get::<percent>(), 100.0);

        let output: &EnvelopeOutput = preset.generator(9).unwrap();
        assert_eq!(output.destination, OutputDestination::Lane1);
    }
}

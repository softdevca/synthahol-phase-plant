//! [Noise Generator](https://kilohearts.com/docs/phase_plant/#noise_generator)
//! can create enharmonic sounds.

use std::any::Any;

use strum_macros::Display;
use uom::si::f32::Frequency;

use super::*;

#[derive(Copy, Clone, Debug, Display, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum NoiseWaveform {
    // The discriminants correspond to the file format.
    Colored = 0,
    KeytrackedStepped = 1,
    KeytrackedSmooth = 2,
}

impl NoiseWaveform {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown noise waveform {id}"),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoiseGenerator {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,

    #[doc(alias = "find_tuning")]
    pub semi_cent: f32,

    pub harmonic: f32,
    pub shift: Frequency,
    pub phase_offset: Ratio,
    pub phase_jitter: Ratio,
    pub level: Ratio,
    pub waveform: NoiseWaveform,

    /// Decibels per octave
    pub slope: Decibels,

    pub stereo: Ratio,

    pub seed_mode: SeedMode,
}

impl NoiseGenerator {
    /// Slope setting for white noise.
    pub const SLOPE_WHITE_NOISE: f32 = 0.0;
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self {
            name: GeneratorMode::NoiseGenerator.name().to_owned(),
            harmonic: 4.0, // Other generators use 1.0 as a default
            ..Self::from(&GeneratorBlock::default())
        }
    }
}

impl From<&GeneratorBlock> for NoiseGenerator {
    fn from(block: &GeneratorBlock) -> Self {
        NoiseGenerator {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            semi_cent: block.fine_tuning,
            harmonic: block.harmonic,
            shift: block.shift,
            phase_offset: block.phase_offset,
            phase_jitter: block.phase_jitter,
            level: block.level,
            waveform: block.noise_waveform,
            slope: block.noise_slope,
            stereo: block.stereo,
            seed_mode: block.seed_mode,
        }
    }
}

impl Generator for NoiseGenerator {
    fn id(&self) -> Option<GeneratorId> {
        Some(self.id)
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::NoiseGenerator
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_nonlinear_filter_generator(&self) -> Option<&NonlinearFilterGenerator> {
        self.downcast_ref::<NonlinearFilterGenerator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::f32::Frequency;

    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn disabled() {
        let preset = read_generator_preset(
            "noise_generator",
            "noise_generator-disabled-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &NoiseGenerator = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn init() {
        for file in &[
            "noise_generator-1.7.0.phaseplant",
            "noise_generator-1.8.0.phaseplant",
            "noise_generator-1.8.13.phaseplant",
            "noise_generator-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("noise_generator", file).unwrap();
            let generator: &NoiseGenerator = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Noise".to_owned());
            assert_eq!(generator.waveform, NoiseWaveform::Colored);
            assert_eq!(generator.level.get::<percent>(), 100.0);
            assert_eq!(generator.semi_cent, 0.0);
            assert_eq!(generator.harmonic, 4.0);
            assert_eq!(generator.shift, Frequency::zero());
            assert_eq!(generator.phase_offset, Ratio::zero());
            assert_eq!(generator.phase_jitter, Ratio::zero());
            assert_relative_eq!(generator.slope.db(), 3.0103, epsilon = 0.0001); // 3.0 db/Oct
            assert_eq!(generator.stereo.get::<percent>(), 0.0);
            assert_eq!(generator.seed_mode, SeedMode::Stable);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_generator_preset(
            "noise_generator",
            "noise_generator-lane3-stereo15-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &NoiseGenerator = preset.generator(1).unwrap();
        assert!(generator.enabled);
        assert_relative_eq!(generator.stereo.get::<percent>(), 15.0);

        let preset = read_generator_preset(
            "noise_generator",
            "noise_generator-stepped-slope2db_oct-stereo25-random-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &NoiseGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.waveform, NoiseWaveform::KeytrackedStepped);
        assert_relative_eq!(generator.slope.db(), 2.0);
        assert_relative_eq!(generator.stereo.get::<percent>(), 25.0);
        assert_eq!(generator.seed_mode, SeedMode::Random);
    }

    #[test]
    fn parts_version_2() {
        let preset = read_generator_preset(
            "noise_generator",
            "noise_generator-seed_random-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &NoiseGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.seed_mode, SeedMode::Random);

        let preset = read_generator_preset(
            "noise_generator",
            "noise_generator-waveform_smooth-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &NoiseGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.waveform, NoiseWaveform::KeytrackedSmooth);
    }

    #[test]
    fn pitch() {
        let preset = read_generator_preset(
            "noise_generator",
            "noise_generator-pitch23-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &NoiseGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.semi_cent, 23.0);
    }
}

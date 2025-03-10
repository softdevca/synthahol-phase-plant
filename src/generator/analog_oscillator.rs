//! [Analog Oscillator](https://kilohearts.com/docs/phase_plant/#analog_oscillator)
//! simulates classic analog waveforms.

use std::any::Any;

use strum_macros::Display;
use uom::si::f32::Frequency;

use super::*;

// TODO: Needs preset name and path

#[derive(Copy, Clone, Debug, Display, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum AnalogWaveform {
    // The discriminants correspond to the file format.
    Saw = 0,
    Square = 1,
    Sine = 3,
    Triangle = 2,
}

impl AnalogWaveform {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown analog waveform {id}"),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalogOscillator {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub tuning: f32,
    pub harmonic: f32,
    pub shift: Frequency,

    /// Percentage of 360 degrees.
    pub phase_offset: Ratio,

    /// Percentage of 360 degrees.
    pub phase_jitter: Ratio,

    /// Amplitude of the waveform. Gain is set in the Out generator.
    pub level: Ratio,
    pub sync_multiplier: f32,
    pub pulse_width: Ratio,
    pub unison: Unison,
    pub waveform: AnalogWaveform,
}

impl Default for AnalogOscillator {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::AnalogOscillator.name().to_owned(),
            ..GeneratorBlock::default()
        })
    }
}

impl From<&GeneratorBlock> for AnalogOscillator {
    fn from(block: &GeneratorBlock) -> Self {
        Self {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            tuning: block.fine_tuning,
            harmonic: block.harmonic,
            shift: block.shift,
            phase_offset: block.phase_offset,
            phase_jitter: block.phase_jitter,
            level: block.level,
            pulse_width: block.pulse_width,
            sync_multiplier: block.sync_multiplier,
            unison: block.unison,
            waveform: block.analog_waveform,
        }
    }
}

impl Generator for AnalogOscillator {
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
        GeneratorMode::AnalogOscillator
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_analog(&self) -> Option<&AnalogOscillator> {
        self.downcast_ref::<AnalogOscillator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::f32::Frequency;
    use uom::si::frequency::hertz;
    use uom::si::ratio::ratio;

    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn default() {
        let generator = AnalogOscillator::default();
        assert!(generator.enabled);
        assert_eq!(generator.name(), "Analog".to_owned());
        assert_eq!(generator.level.get::<percent>(), 100.0);
        assert_eq!(generator.tuning, 0.0);
        assert_eq!(generator.harmonic, 1.0);
        assert_eq!(generator.shift, Frequency::zero());
        assert_eq!(generator.phase_offset, Ratio::zero());
        assert_eq!(generator.phase_jitter, Ratio::zero());
        assert_eq!(generator.waveform, AnalogWaveform::Saw);
        assert_eq!(generator.sync_multiplier, 1.0);
        assert_eq!(generator.pulse_width.get::<percent>(), 50.0);
        assert_eq!(generator.unison, Unison::default());
    }

    #[test]
    fn init() {
        for file in &[
            "analog_oscillator-1.7.0.phaseplant",
            "analog_oscillator-1.8.13.phaseplant",
            "analog_oscillator-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("analog_oscillator", file).unwrap();
            let generator: &AnalogOscillator = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Analog".to_owned());
            assert_eq!(generator.level.get::<percent>(), 100.0);
            assert_eq!(generator.tuning, 0.0);
            assert_eq!(generator.harmonic, 1.0);
            assert_eq!(generator.shift, Frequency::zero());
            assert_eq!(generator.phase_offset, Ratio::zero());
            assert_eq!(generator.phase_jitter, Ratio::zero());
            assert_eq!(generator.waveform, AnalogWaveform::Saw);
            assert_eq!(generator.sync_multiplier, 1.0);
            assert_eq!(generator.pulse_width.get::<percent>(), 50.0);

            // In Phase Plant 1.8.5 the default Unison voices changed from 1 to 4.
            if preset
                .format_version
                .is_at_least(&PhasePlantRelease::V1_8_5.format_version())
            {
                assert_eq!(generator.unison, Default::default());
            } else {
                assert_eq!(
                    generator.unison,
                    Unison {
                        voices: 1,
                        ..Default::default()
                    }
                );
            }
        }
    }

    #[test]
    fn disabled() {
        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-disabled-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn parts() {
        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-level90%-semi11.5-harmonic3-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert_eq!(generator.level.get::<percent>(), 90.0);
        assert_eq!(generator.tuning, 11.5);
        assert_eq!(generator.harmonic, 3.0);

        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-shift--99hz-phase_offset15_jitter20-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert_relative_eq!(generator.shift.get::<hertz>(), -99.0, epsilon = 0.001);
        assert_relative_eq!(generator.phase_offset.get::<ratio>(), 15.0 / 360.0);
        assert_relative_eq!(generator.phase_jitter.get::<ratio>(), 20.0 / 360.0);

        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-sine-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert_eq!(generator.waveform, AnalogWaveform::Sine);

        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-sync3-pw25%-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert_eq!(generator.sync_multiplier, 3.0);
        assert_eq!(generator.pulse_width.get::<percent>(), 25.0);
    }

    #[test]
    fn unison() {
        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-unison-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert_eq!(
            generator.unison,
            Unison {
                enabled: true,
                ..Default::default()
            }
        );

        let preset = read_generator_preset(
            "analog_oscillator",
            "analog_oscillator-unison-octaves-balance35%-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &AnalogOscillator = preset.generator(1).unwrap();
        assert!(generator.unison.enabled);
        assert_eq!(generator.unison.mode, UnisonMode::Octaves);
        assert_relative_eq!(generator.unison.bias.get::<percent>(), 35.0);
    }
}

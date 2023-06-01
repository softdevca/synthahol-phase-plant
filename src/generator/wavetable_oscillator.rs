//! [Wavetable Oscillator](https://kilohearts.com/docs/phase_plant/#wavetable_oscillator)
//! plays back a wavetable that has 256 frames where each frames is a waveform
//! containing 2048 samples.

use std::any::Any;

use uom::si::f32::Frequency;

use super::*;

// const SAMPLE_COUNT: usize = 2048;
// const FRAME_COUNT: usize = 256;
// = 524288 samples

// TODO: Needs preset name and path

#[derive(Clone, Debug, PartialEq)]
pub struct WavetableOscillator {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub tuning: f32,
    pub harmonic: f32,
    pub shift: Frequency,
    pub phase_offset: Ratio,
    pub phase_jitter: Ratio,
    pub level: f32,
    pub frame: f32,
    pub band_limit: f32,
    pub unison: Unison,
    pub wavetable_contents: Vec<u8>,
    pub wavetable_edited: bool,
    pub wavetable_name: Option<String>,
    pub wavetable_path: Option<String>,
}

impl Default for WavetableOscillator {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::WavetableOscillator.name().to_owned(),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for WavetableOscillator {
    fn from(block: &GeneratorBlock) -> Self {
        WavetableOscillator {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            tuning: block.fine_tuning,
            harmonic: block.harmonic,
            shift: block.shift,
            phase_offset: block.phase_offset,
            phase_jitter: block.phase_jitter,
            level: block.level,
            frame: block.wavetable_frame,
            band_limit: block.band_limit,
            unison: block.unison,
            wavetable_contents: block.wavetable_contents.clone(),
            wavetable_edited: block.wavetable_edited,
            wavetable_name: block.wavetable_name.clone(),
            wavetable_path: block.wavetable_path.clone(),
        }
    }
}

impl Generator for WavetableOscillator {
    fn id(&self) -> Option<GeneratorId> {
        Some(self.id)
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::WavetableOscillator
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_wavetable(&self) -> Option<&WavetableOscillator> {
        self.downcast_ref::<WavetableOscillator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::f32::Frequency;

    use crate::test::read_generator_preset;

    use super::WavetableOscillator;
    use super::*;

    #[test]
    fn init() {
        for file in &[
            "wavetable_oscillator-1.7.0.phaseplant",
            "wavetable_oscillator-1.7.7.phaseplant",
            "wavetable_oscillator-1.8.0.phaseplant",
            "wavetable_oscillator-1.8.5.phaseplant",
            "wavetable_oscillator-1.8.17.phaseplant",
            "wavetable_oscillator-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("wavetable_oscillator", file).unwrap();
            let generator: &WavetableOscillator = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Wavetable".to_owned());
            assert_eq!(generator.level, 1.0);
            assert_eq!(generator.tuning, 0.0);
            assert_eq!(generator.harmonic, 1.0);
            assert_eq!(generator.shift, Frequency::zero());
            assert_eq!(generator.phase_offset, Ratio::zero());
            assert_eq!(generator.phase_jitter, Ratio::zero());
            assert_eq!(generator.frame, 0.0);
            assert_relative_eq!(generator.band_limit, 22050.0);
            assert_eq!(
                generator.wavetable_name,
                Some("Default Wavetable".to_owned())
            );

            if preset
                .format_version
                .is_at_least(&PhasePlantRelease::V1_8_0.format_version())
            {
                assert_eq!(
                    generator.wavetable_path,
                    Some("factory/Morphs/Default Wavetable.flac".to_owned())
                );
            }

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
            "wavetable_oscillator",
            "wavetable_oscillator-disabled-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-brass-edited-1.8.17.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert_eq!(generator.wavetable_name, Some("BrassEdited".to_owned()));
        assert_eq!(
            generator.wavetable_path,
            Some("user/BrassEdited.flac".to_owned())
        );

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-frame33-bandlimit8k-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert_eq!(generator.frame, 32.0);
        assert_relative_eq!(generator.band_limit, 8000.0, epsilon = 0.001);

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-newspeak-1.8.17.phaseplant",
        )
        .unwrap();
        let _generator: &WavetableOscillator = preset.generator(1).unwrap();
        #[cfg(feature = "")]
        {
            // FIXME: Disabled until data blocks figured out.  BrassEdited
            assert_eq!(
                generator.wavetable_path,
                Some("factory/Morphs/Saw to Sine.flac".to_owned())
            );
        }

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-saw_to_sine-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert_eq!(generator.wavetable_name, Some("Saw to Sine".to_owned()));
        #[cfg(feature = "")]
        {
            // FIXME: Disabled until data blocks figured out
            assert_eq!(
                generator.wavetable_path,
                Some("factory/Morphs/Saw to Sine.flac".to_owned())
            );
        }

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-unison-1.8.14.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert!(generator.unison.enabled);

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-unison-blend25-bias10-1.8.14.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert!(generator.unison.enabled);
        assert_relative_eq!(generator.unison.blend, 0.25);
        assert_relative_eq!(generator.unison.bias, 0.1);

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-unison-detune15-spread50-1.8.14.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert!(generator.unison.enabled);
        assert_relative_eq!(generator.unison.detune, 15.0);
        assert_relative_eq!(generator.unison.spread, 0.5);

        let preset = read_generator_preset(
            "wavetable_oscillator",
            "wavetable_oscillator-unison2-fifths-1.8.14.phaseplant",
        )
        .unwrap();
        let generator: &WavetableOscillator = preset.generator(1).unwrap();
        assert!(generator.unison.enabled);
        assert_eq!(generator.unison.voices, 2);
        assert_eq!(generator.unison.mode, UnisonMode::Fifths);
    }
}

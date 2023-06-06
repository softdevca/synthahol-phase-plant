//! [Distortion Effect](https://kilohearts.com/docs/phase_plant/#distortion_effect)
//! generator

use std::any::Any;

use crate::effect::Distortion;
use crate::generator::{Generator, GeneratorMode};

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DistortionEffect {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub effect: Distortion,
}

impl Default for DistortionEffect {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: "Distortion".to_owned(),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for DistortionEffect {
    fn from(block: &GeneratorBlock) -> Self {
        Self {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            effect: block.distortion_effect.clone(),
        }
    }
}

impl Generator for DistortionEffect {
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
        GeneratorMode::DistortionEffect
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_distortion(&self) -> Option<&DistortionEffect> {
        self.downcast_ref::<DistortionEffect>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::DistortionMode;
    use crate::generator::Generator;
    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn disabled() {
        let preset = read_generator_preset(
            "distortion_effect",
            "distortion_effect-disabled-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &DistortionEffect = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn foldback_and_bias() {
        let preset = read_generator_preset(
            "distortion_effect",
            "distortion_effect-foldback-bias25%-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &DistortionEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.mode, DistortionMode::Foldback);
        assert_eq!(generator.effect.bias.get::<percent>(), 25.0);
    }

    #[test]
    fn hard_clip_and_mix() {
        let preset = read_generator_preset(
            "distortion_effect",
            "distortion_effect-hard_clip-mix80%-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &DistortionEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.mode, DistortionMode::HardClip);
        assert_eq!(generator.effect.mix.get::<percent>(), 80.0);
    }

    #[test]
    fn init() {
        for file in &[
            "distortion_effect-1.7.0.phaseplant",
            "distortion_effect-1.8.13.phaseplant",
            "distortion_effect-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("distortion_effect", file).unwrap();
            let generator: &DistortionEffect = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Distortion".to_owned());
            assert_eq!(generator.effect.mode, DistortionMode::Overdrive);
            println!("DB: {:?}", generator.effect.drive.db());
            assert_relative_eq!(generator.effect.drive.db(), 12.04, epsilon = 0.01);
            assert_relative_eq!(generator.effect.bias.get::<percent>(), 0.0);
            assert_relative_eq!(generator.effect.spread.get::<percent>(), 0.0);
            assert_relative_eq!(generator.effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn quantize_and_sideband() {
        let preset = read_generator_preset(
            "distortion_effect",
            "distortion_effect-quantize-send_to_sideband-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &DistortionEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.mode, DistortionMode::Quantize);
        let envelope_output: &EnvelopeOutput = preset.generator(2).unwrap();
        assert_eq!(envelope_output.destination, OutputDestination::Sideband);
    }

    #[test]
    fn saturate_and_drive() {
        let preset = read_generator_preset(
            "distortion_effect",
            "distortion_effect-saturate-drive10db-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &DistortionEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.mode, DistortionMode::Saturate);
        assert_eq!(generator.effect.drive.db(), 10.0);
    }

    #[test]
    fn sine_and_spread() {
        let preset = read_generator_preset(
            "distortion_effect",
            "distortion_effect-sine-spread11%-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &DistortionEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.mode, DistortionMode::Sine);
        assert_eq!(generator.effect.spread.get::<percent>(), 11.0);
    }
}

//! The [Filter Effect](https://kilohearts.com/docs/phase_plant/#filter_effect)
//! generator is an effect module that works like the [Filter] Snapin effect.

use std::any::Any;

use crate::effect::Filter;

// FIXME: Slope control added in 1.8.0
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FilterEffect {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub effect: Filter,
}

impl Default for FilterEffect {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::FilterEffect.name().to_owned(),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for FilterEffect {
    fn from(block: &GeneratorBlock) -> Self {
        FilterEffect {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            effect: block.filter_effect.clone(),
        }
    }
}

impl Generator for FilterEffect {
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
        GeneratorMode::FilterEffect
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_filter(&self) -> Option<&FilterEffect> {
        self.downcast_ref::<FilterEffect>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::FilterMode;
    use crate::test::read_generator_preset;
    use crate::Decibels;

    use super::*;

    #[test]
    fn defaults() {
        let generator = FilterEffect::default();
        assert_eq!(generator.effect.filter_mode, FilterMode::LowPass);
        assert_relative_eq!(generator.effect.cutoff_frequency, 440.0, epsilon = 0.0001);
        assert_relative_eq!(generator.effect.q, 0.707, epsilon = 0.0001);
        assert_eq!(generator.effect.gain, Decibels::ZERO);
        assert_eq!(generator.effect.slope, 1);
    }

    #[test]
    fn disabled() {
        let preset =
            read_generator_preset("filter_effect", "filter_effect-disabled-1.8.13.phaseplant")
                .unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn init() {
        for file in &[
            "filter_effect-1.7.0.phaseplant",
            "filter_effect-1.8.13.phaseplant",
            "filter_effect-2.0.16.phaseplant",
        ] {
            let preset = read_generator_preset("filter_effect", file).unwrap();
            let generator: &FilterEffect = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Filter".to_owned());
            assert_eq!(generator.effect.filter_mode, FilterMode::LowPass);
            assert_relative_eq!(generator.effect.cutoff_frequency, 440.0, epsilon = 0.0001);
            assert_relative_eq!(generator.effect.q, 0.707, epsilon = 0.0001);
            assert_eq!(generator.effect.gain, Decibels::ZERO);
            assert_eq!(generator.effect.slope, 1);
        }
    }

    #[test]
    fn parts() {
        let preset =
            read_generator_preset("filter_effect", "filter_effect-1.8.13.phaseplant").unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert!(generator.enabled);

        let preset = read_generator_preset(
            "filter_effect",
            "filter_effect-bandpass-cutoff220hz-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.filter_mode, FilterMode::BandPass);
        assert_relative_eq!(generator.effect.cutoff_frequency, 220.0, epsilon = 0.0001);

        // FIXME: THIS IS AN *OUT* SLOPE ADSR
        let preset = read_generator_preset(
            "filter_effect",
            "filter_effect-high_shelf-slope3-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.filter_mode, FilterMode::HighShelf);
        assert_eq!(generator.effect.slope, 3);

        let preset = read_generator_preset(
            "filter_effect",
            "filter_effect-low_shelf-gain1.5db-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.filter_mode, FilterMode::LowShelf);
        assert_relative_eq!(generator.effect.gain.db(), 1.5, epsilon = 0.0001);

        let preset = read_generator_preset(
            "filter_effect",
            "filter_effect-notch-q2.220-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.filter_mode, FilterMode::Notch);
        assert_relative_eq!(generator.effect.q, 2.22, epsilon = 0.0001);

        let preset = read_generator_preset(
            "filter_effect",
            "filter_effect-peak-slope3-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &FilterEffect = preset.generator(1).unwrap();
        assert_eq!(generator.effect.filter_mode, FilterMode::Peak);
        assert_eq!(generator.effect.slope, 3);
    }
}

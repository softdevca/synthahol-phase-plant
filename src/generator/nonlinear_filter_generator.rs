//! Nonlinear Filter Generator is the [nonlinear filter](NonlinearFilter), but in the
//! generator section.
//!
//! The generator was added in Phase Plant 2.1.1

use std::any::Any;

use crate::effect::NonlinearFilter;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NonlinearFilterGenerator {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub effect: NonlinearFilter,
}

impl Default for NonlinearFilterGenerator {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::NonlinearFilterGenerator.name().to_owned(),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for NonlinearFilterGenerator {
    fn from(block: &GeneratorBlock) -> Self {
        NonlinearFilterGenerator {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            effect: block.nonlinear_filter_effect.clone(),
        }
    }
}

impl Generator for NonlinearFilterGenerator {
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
        GeneratorMode::NonlinearFilterGenerator
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_nonlinear_filter(&self) -> Option<&NonlinearFilterGenerator> {
        self.downcast_ref::<NonlinearFilterGenerator>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::effect::{FilterMode, NonlinearFilterMode};
    use crate::test::read_generator_preset;
    use uom::si::f32::Frequency;
    use uom::si::frequency::hertz;

    #[test]
    fn init() {
        for file in &["nonlinear_filter_generator-2.1.1.phaseplant"] {
            let preset = read_generator_preset("nonlinear_filter_generator", file).unwrap();
            let generator: &NonlinearFilterGenerator = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Nonlinear Filter".to_owned());
            let effect = &generator.effect;
            assert_eq!(effect, &Default::default());
        }
    }

    #[ignore]
    #[test]
    fn parts() {
        let preset = read_generator_preset(
            "nonlinear_filter_generator",
            "nonlinear_filter_generator-all_pass-disabled-2.1.3.phaseplant",
        )
        .unwrap();
        let generator: &NonlinearFilterGenerator = preset.generator(1).unwrap();
        assert!(!generator.enabled);
        // let effect = &generator.effect;
        // FIXME: All pass mode
        // assert_eq!(effect.filter_mode, FilterMode::A)

        let preset = read_generator_preset(
            "nonlinear_filter_generator",
            "nonlinear_filter_generator-band_pass-q1.5-warm-2.1.3.phaseplant",
        )
        .unwrap();
        let generator: &NonlinearFilterGenerator = preset.generator(1).unwrap();
        assert!(generator.enabled);
        let effect = &generator.effect;
        assert_eq!(effect.filter_mode, FilterMode::BandPass);
        assert_eq!(effect.mode, NonlinearFilterMode::Warm);
        assert_eq!(effect.q, 1.5);

        let preset = read_generator_preset(
            "nonlinear_filter_generator",
            "nonlinear_filter_generator-high_pass-drive50-2.1.3.phaseplant",
        )
        .unwrap();
        let generator: &NonlinearFilterGenerator = preset.generator(1).unwrap();
        assert!(generator.enabled);
        let effect = &generator.effect;
        assert_eq!(effect.filter_mode, FilterMode::HighPass);
        assert_eq!(effect.drive, 0.5);

        let preset = read_generator_preset(
            "nonlinear_filter_generator",
            "nonlinear_filter_generator-notch-cutoff1000-2.1.3.phaseplant",
        )
        .unwrap();
        let generator: &NonlinearFilterGenerator = preset.generator(1).unwrap();
        let effect = &generator.effect;
        assert_eq!(effect.filter_mode, FilterMode::Notch);
        assert_eq!(effect.cutoff, Frequency::new::<hertz>(1000.0));
    }
}

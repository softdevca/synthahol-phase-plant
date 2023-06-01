//! The [Mix Routing](https://kilohearts.com/docs/phase_plant/#mix_routing)
//! generator can combine generators above it and adjust the gain.

use std::any::Any;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MixRouting {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub level: f32,
    pub invert: bool,
}

impl Default for MixRouting {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::MixRouting.name().to_owned(),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for MixRouting {
    fn from(block: &GeneratorBlock) -> Self {
        MixRouting {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            level: block.mix_level,
            invert: block.invert,
        }
    }
}

impl Generator for MixRouting {
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
        GeneratorMode::MixRouting
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_mix(&self) -> Option<&MixRouting> {
        self.downcast_ref::<MixRouting>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &[
            "mix_routing-1.7.0.phaseplant",
            "mix_routing-1.8.13.phaseplant",
            "mix_routing-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("mix_routing", file).unwrap();
            let generator: &MixRouting = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name(), "Mix".to_owned());
            assert_relative_eq!(generator.level, 1.0);
            assert!(!generator.invert);
        }
    }

    #[test]
    fn disabled() {
        let preset =
            read_generator_preset("mix_routing", "mix_routing-disabled-1.8.16.phaseplant").unwrap();
        let generator: &MixRouting = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn parts() {
        let preset = read_generator_preset(
            "mix_routing",
            "mix_routing-level80-invert-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &MixRouting = preset.generator(1).unwrap();
        assert!(generator.enabled);
        assert!(generator.invert);
        assert_relative_eq!(generator.level, 0.8);
    }
}

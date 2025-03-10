//! The [Aux Routing](https://kilohearts.com/docs/phase_plant/#aux_routing)
//! generator combines signals and adjust the gain.
//!
//! Unlike the [Mix Routing](generator::mix_routing), the generators above are not
//! automatically included.

use std::any::Any;

use crate::generator::{Generator, GeneratorId, GeneratorMode};
use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AuxRouting {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub invert: bool,
    pub level: Ratio,
}

impl Default for AuxRouting {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::AuxRouting.name().to_owned(),
            ..GeneratorBlock::default()
        })
    }
}

impl From<&GeneratorBlock> for AuxRouting {
    fn from(block: &GeneratorBlock) -> Self {
        Self {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            invert: block.invert,
            level: block.mix_level,
        }
    }
}

impl Generator for AuxRouting {
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
        GeneratorMode::AuxRouting
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_auxiliary(&self) -> Option<&AuxRouting> {
        self.downcast_ref::<AuxRouting>()
    }
}

#[cfg(test)]
mod test {
    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn default() {
        let generator = AuxRouting::default();
        assert!(generator.enabled);
        assert_eq!(generator.name, "Aux".to_owned());
        assert_eq!(generator.level.get::<percent>(), 100.0);
        assert!(!generator.invert);
    }

    #[test]
    fn init() {
        for file in &[
            "aux_routing-1.7.0.phaseplant",
            "aux_routing-1.8.13.phaseplant",
            "aux_routing-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("aux_routing", file).unwrap();
            let generator: &AuxRouting = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert_eq!(generator.name, "Aux".to_owned());
            assert_eq!(generator.level.get::<percent>(), 100.0);
            assert!(!generator.invert);
        }
    }

    #[test]
    fn disabled() {
        let preset =
            read_generator_preset("aux_routing", "aux_routing-disabled-1.8.16.phaseplant").unwrap();
        let generator: &AuxRouting = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn parts() {
        let preset = read_generator_preset(
            "aux_routing",
            "aux_routing-level25-invert-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &AuxRouting = preset.generator(1).unwrap();
        assert!(generator.enabled);
        assert_eq!(generator.level.get::<percent>(), 25.0);
        assert!(generator.invert);
    }
}

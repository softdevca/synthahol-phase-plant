//! A [Generator Group](https://kilohearts.com/docs/phase_plant/#generator_groups)
//! is a collection of generator modules that controls the routing of signals.

use std::any::Any;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub id: GeneratorId,
    pub enabled: bool,
    pub minimized: bool,
    pub name: String,
}

impl Group {
    pub const MAX_NAME_LENGTH: usize = 45; // From Phase Plant 1.8.20
}

impl Default for Group {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::Group.name().to_owned(),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for Group {
    fn from(block: &GeneratorBlock) -> Self {
        Group {
            id: block.id,
            enabled: block.enabled,
            minimized: block.minimized,
            name: block.name.clone(),
        }
    }
}

impl Generator for Group {
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
        GeneratorMode::Group
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_group(&self) -> Option<&Group> {
        self.downcast_ref::<Group>()
    }
}

#[cfg(test)]
mod test {
    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn disabled() {
        let preset = read_generator_preset("group", "group-disabled-1.8.13.phaseplant").unwrap();
        let generator: &Group = preset.generator(0).unwrap();
        assert!(!generator.enabled);
        assert!(!generator.minimized);
    }

    #[test]
    fn init() {
        for file in &[
            "group-1.7.0.phaseplant",
            "group-1.8.13.phaseplant",
            "group-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("group", file).unwrap();
            let generator: &Group = preset.generator(0).unwrap();
            assert!(generator.enabled);
            assert!(!generator.minimized);
            assert_eq!(generator.name(), "Group".to_owned());
            assert_eq!(generator, &Group::default());
        }
    }

    #[test]
    fn minimized() {
        let preset = read_generator_preset("group", "group-minimized-1.8.13.phaseplant").unwrap();
        let generator: &Group = preset.generator(0).unwrap();
        assert!(generator.enabled);
        assert!(generator.minimized);
    }

    #[test]
    fn name() {
        let preset = read_generator_preset("group", "group-named-1.8.20.phaseplant").unwrap();
        let generator: &Group = preset.generator(0).unwrap();
        assert!(generator.enabled);
        assert!(!generator.minimized);
        assert_eq!(generator.name(), "Slartibartfast");
    }

    #[test]
    fn with_out() {
        let preset = read_generator_preset("group", "group-with-out-1.8.13.phaseplant").unwrap();
        let generator: &Group = preset.generator(0).unwrap();
        assert!(generator.enabled);
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert!(generator.enabled);
    }
}

//! Group modulator
//!
//! The Group modulator was added in Phase Plant version 2.

use std::any::Any;

use crate::ModulatorBlock;
use crate::modulator::{Modulator, ModulatorMode};

#[derive(Debug, Default, PartialEq)]
pub struct Group {
    pub name: Option<String>,
}

impl Modulator for Group {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Group
    }
}

#[cfg(test)]
mod test {
    use crate::modulator::{LfoModulator, ModulatorId};
    use crate::test::read_modulator_preset;

    use super::*;

    /// Three groups that each contain an LFO.
    #[test]
    fn contains() {
        let preset =
            read_modulator_preset("group", "group-3contains_lfo-2.0.14.phaseplant").unwrap();
        for idx in (0..6).step_by(2) {
            let container = preset.modulator_container(idx).unwrap();
            assert_eq!(container.id, idx as ModulatorId);
            let modulator: &LfoModulator = preset.modulator(idx + 1).unwrap();
            assert_eq!(modulator, &LfoModulator::default());
        }
    }

    #[test]
    fn default() {
        let modulator = Group::default();
        assert_eq!(modulator.name, None);
    }

    #[test]
    fn disabled() {
        let preset = read_modulator_preset("group", "group-disabled-2.0.12.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(!container.enabled);
        assert!(!container.minimized);
    }

    #[test]
    fn init() {
        for file in &["group-2.0.12.phaseplant", "group-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("group", file).unwrap();
            let container = preset.modulator_container(0).unwrap();
            assert_eq!(container.id, 0);
            let modulator: &Group = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Group::default());
        }
    }

    #[test]
    fn minimized() {
        let preset = read_modulator_preset("group", "group-minimized-2.0.12.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(container.minimized);
    }

    #[test]
    fn name() {
        let preset = read_modulator_preset("group", "group-new-name-2.0.12.phaseplant").unwrap();
        let modulator: &Group = preset.modulator(0).unwrap();
        assert_eq!(modulator.name, Some("New Name".to_owned()));
    }
}

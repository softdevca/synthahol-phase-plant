//! [Note Gate Modulator](https://kilohearts.com/docs/modulation#note_gate)

use std::any::Any;

use crate::modulator::{Modulator, ModulatorMode, OutputRange};

use super::*;

#[derive(Debug, PartialEq)]
pub struct NoteGateModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
}

impl Modulator for NoteGateModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::NoteGate
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &["note_gate-2.0.12.phaseplant", "note_gate-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("note_gate", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &NoteGateModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator.output_range, OutputRange::Unipolar);
            assert_relative_eq!((modulator.depth.get::<percent>()), 100.0);
        }
    }
}

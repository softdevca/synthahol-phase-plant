//! [MIDI CC Modulator](https://kilohearts.com/docs/modulation#midi_cc)

use std::any::Any;

use uom::si::ratio::ratio;

use super::*;

#[derive(Debug, PartialEq)]
pub struct MidiCcModulator {
    pub output_range: OutputRange,
    pub depth: Ratio,
    pub controller_slot: Option<u32>,
}

impl Default for MidiCcModulator {
    fn default() -> Self {
        Self {
            output_range: OutputRange::Unipolar,
            depth: Ratio::new::<ratio>(1.0),
            controller_slot: None,
        }
    }
}

impl Modulator for MidiCcModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::MidiCc
    }
}

#[cfg(test)]
mod test {
    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &["midi_cc-2.0.12.phaseplant", "midi_cc-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("midi_cc", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            let modulator: &MidiCcModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }

    #[test]
    fn controller_number() {
        let preset = read_modulator_preset("midi_cc", "midi_cc-slot25-2.1.0.phaseplant").unwrap();
        let modulator: &MidiCcModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.controller_slot, Some(25));
    }
}

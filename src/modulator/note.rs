//! [Note Modulator](https://kilohearts.com/docs/modulation#note)

use std::any::Any;
use std::ops::RangeInclusive;

use crate::modulator::{Modulator, ModulatorMode, OutputRange};

use super::*;

#[derive(Debug, PartialEq)]
pub struct NoteModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
    pub root_note: u32,

    /// Measured in number of notes.
    pub note_range: u32,
}

impl NoteModulator {
    /// Legal values for [note_range](Self::note_range).
    pub const NOTE_RANGE: RangeInclusive<u8> = 12..=120;
}

impl Modulator for NoteModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Note
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use music_note::midi;
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &["note-1.8.13.phaseplant", "note-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("note", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &NoteModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator.output_range, OutputRange::Bipolar);
            assert_eq!(modulator.root_note, midi!(A, 4).into_byte() as u32);
            assert_eq!(modulator.note_range, 120);
            assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn parts() {
        let preset =
            read_modulator_preset("note", "note-center_d5-range_12-disabled-2.1.0.phaseplant")
                .unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(!container.enabled);
        assert!(!container.minimized);
        let modulator: &NoteModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.note_range, 12);
        assert_eq!(modulator.root_note, midi!(D, 5).into_byte() as u32);

        let preset =
            read_modulator_preset("note", "note-depth50-inverted-2.1.0.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(!container.minimized);
        let modulator: &NoteModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.output_range, OutputRange::Inverted);
        assert_relative_eq!(modulator.depth.get::<percent>(), 50.0);
    }
}

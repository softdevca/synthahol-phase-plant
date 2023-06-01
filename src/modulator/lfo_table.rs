//! [LFO Table Modulator](https://kilohearts.com/docs/modulation#lfo_table)
//!
//! The LFO Table Modulator was added to Phase Plant in version 2.0.

use std::any::Any;

use uom::si::f32::Frequency;
use uom::si::frequency::hertz;
use uom::si::ratio::{percent, ratio};

use crate::generator::LoopMode;
use crate::modulator::{Modulator, ModulatorMode, NoteTriggerMode, OutputRange};
use crate::*;

#[derive(Debug, PartialEq)]
pub struct LfoTableModulator {
    pub output_range: OutputRange,
    pub depth: Ratio,

    /// In the Phase Plant interface the rate is specified as time. It is
    /// converted to the rate frequency on save and load to avoid duplication.
    pub rate: Rate,

    pub loop_mode: LoopMode,

    pub note_trigger_mode: NoteTriggerMode,
    pub trigger_threshold: f32,

    // Portion of 360 degrees.
    pub phase_offset: Ratio,

    /// 0.05% to 20.0%
    pub smooth: Ratio,
    pub frame: f32,

    // Wavetable
    pub wavetable_contents: Vec<u8>,
    pub wavetable_name: Option<String>,
    pub wavetable_path: Option<String>,
}

impl Default for LfoTableModulator {
    fn default() -> Self {
        Self {
            output_range: OutputRange::Unipolar,
            depth: Ratio::new::<ratio>(1.0),
            rate: Rate {
                sync: false,
                frequency: Frequency::new::<hertz>(1.0),
                numerator: 4,
                denominator: NoteValue::Sixteenth,
            },
            loop_mode: LoopMode::Infinite,
            note_trigger_mode: NoteTriggerMode::Auto,
            trigger_threshold: 0.5,
            phase_offset: Ratio::zero(),
            smooth: Ratio::new::<percent>(0.05),
            frame: 0.0,
            wavetable_contents: Vec::new(),
            wavetable_name: None,
            wavetable_path: None,
        }
    }
}

impl Modulator for LfoTableModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::LfoTable
    }
}

impl LfoTableModulator {
    // Enable if UOM gets const fn's.
    // pub const MAX_SMOOTH: Ratio = Ratio::new::<percent>(20.0);
    // pub const MIN_SMOOTH: Ratio = Ratio::new::<percent>(0.05);

    pub fn phase_offset_degrees(&self) -> f32 {
        self.phase_offset.get::<ratio>() * 360.0
    }
}

impl dyn Modulator {
    #[must_use]
    pub fn as_lfo_table(&self) -> Option<&LfoTableModulator> {
        self.downcast_ref::<LfoTableModulator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::frequency::hertz;
    use uom::si::ratio::percent;

    use crate::modulator::{OutputRange, GROUP_ID_NONE};
    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &[
            "lfo_table-2.0.0.phaseplant",
            "lfo_table-2.0.12.phaseplant",
            "lfo_table-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("lfo_table", file).unwrap();
            let container = preset.modulator_container(0).unwrap();
            assert_eq!(container.id, 0);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator.output_range, OutputRange::Unipolar);
            assert_eq!(modulator.depth.get::<percent>(), 100.0);
            assert!(!modulator.rate.sync);
            assert_eq!(modulator.loop_mode, LoopMode::Infinite);
            assert_eq!(modulator.rate.frequency.get::<hertz>(), 1.0);
            assert_eq!(modulator.rate.numerator, 4);
            assert_eq!(modulator.rate.denominator, NoteValue::Sixteenth);
            assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Auto);
            assert_eq!(modulator.trigger_threshold, 0.5);
            assert_eq!(modulator.phase_offset, Ratio::zero());
            assert_relative_eq!(modulator.frame, 0.0);
            assert_relative_eq!(modulator.smooth.get::<percent>(), 0.05);
        }
    }

    #[test]
    fn parts() {
        let preset =
            read_modulator_preset("lfo_table", "lfo_table-frame15-phase45-2.0.12.phaseplant")
                .unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.frame, 14.0);
        assert_relative_eq!(modulator.phase_offset_degrees(), 45.0);

        let preset = read_modulator_preset(
            "lfo_table",
            "lfo_table-frame10-smooth20-phase180-2.0.0.phaseplant",
        )
        .unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.smooth.get::<percent>(), 0.05);
        assert_relative_eq!(modulator.frame, 9.0);
        assert_relative_eq!(modulator.phase_offset_degrees(), 180.0);

        let preset = read_modulator_preset(
            "lfo_table",
            "lfo_table-ping_pong-trigger_never-threshold75-2.0.0.phaseplant",
        )
        .unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.loop_mode, LoopMode::PingPong);
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Never);
        assert_relative_eq!(modulator.trigger_threshold, 0.75);
    }

    #[test]
    fn rate() {
        let preset =
            read_modulator_preset("lfo_table", "lfo_table-rate_5hz-2.0.12.phaseplant").unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert!(!modulator.rate.sync);
        assert_relative_eq!(
            modulator.rate.frequency.get::<hertz>(),
            5.0,
            epsilon = 0.0001
        );

        let preset =
            read_modulator_preset("lfo_table", "lfo_table-rate_5_8-bipolar-2.0.0.phaseplant")
                .unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert!(modulator.rate.sync);
        assert_eq!(modulator.rate.numerator, 5);
        assert_eq!(modulator.rate.denominator, NoteValue::Eighth);
        assert_eq!(modulator.output_range, OutputRange::Bipolar);
    }

    #[test]
    fn wavetable() {
        // Factory wavetable.
        let preset =
            read_modulator_preset("lfo_table", "lfo_table-wavetable_plucker-2.0.0.phaseplant")
                .unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.wavetable_name, Some("Plucker".to_owned()));
        assert_eq!(
            modulator.wavetable_path,
            Some("factory/LFOs/Simple/Plucker.flac".to_owned())
        );
        assert!(modulator.wavetable_contents.is_empty());

        // Custom wavetable.
        let preset =
            read_modulator_preset("lfo_table", "lfo_table-wavetable_custom-2.1.0.phaseplant")
                .unwrap();
        let modulator: &LfoTableModulator = preset.modulator(0).unwrap();
        assert!(modulator.wavetable_name.is_none());
        assert!(modulator.wavetable_path.is_none());
        assert_eq!(modulator.wavetable_contents.len(), 136191);
        assert_eq!(&modulator.wavetable_contents[0..4], b"fLaC");
    }
}

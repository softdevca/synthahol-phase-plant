//! [Low frequency oscillator (LFO) modulator](https://kilohearts.com/docs/modulation#lfo)

use std::any::Any;

use strum_macros::FromRepr;
use uom::si::f32::Frequency;
use uom::si::frequency::hertz;
use uom::si::ratio::ratio;

use crate::generator::LoopMode;
use crate::modulator::{Modulator, ModulatorMode, OutputRange};
use crate::point::{CurvePoint, CurvePointMode};
use crate::*;

/// [Triggering](https://kilohearts.com/docs/modulation#triggering)
#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum NoteTriggerMode {
    // The discriminants correspond to the file format. They are in the order
    // they are in the Phase Plant interface.
    Auto = 3,
    Never = 0,
    #[doc(alias = "NoteOn")]
    Always = 2,
    Legato = 1,
}

impl NoteTriggerMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown note trigger mode {id}"),
            )
        })
    }
}

impl Display for NoteTriggerMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            NoteTriggerMode::Auto => "Auto",
            NoteTriggerMode::Never => "Never",
            NoteTriggerMode::Always => "Always",
            NoteTriggerMode::Legato => "Legato",
        };
        f.write_str(msg)
    }
}

#[derive(Debug, PartialEq)]
pub struct LfoModulator {
    pub output_range: OutputRange,
    pub depth: Ratio,
    pub loop_mode: LoopMode,
    pub rate: Rate,

    /// Ranges from -1.0..=1.0
    pub trigger_threshold: Ratio,
    pub note_trigger_mode: NoteTriggerMode,

    pub phase_offset: Ratio,

    pub shape: Vec<CurvePoint>,
    pub shape_name: Option<String>,
    pub shape_path: Option<String>,
    pub shape_edited: bool,
}

impl Default for LfoModulator {
    fn default() -> Self {
        Self {
            // The default output range changed from Unipolar in Phase Plant 1
            // to bipolar in Phase Plant 2.
            output_range: OutputRange::Unipolar,
            depth: Ratio::new::<ratio>(1.0),

            loop_mode: LoopMode::Infinite,
            rate: Rate {
                sync: false,
                frequency: Frequency::new::<hertz>(1.0),
                numerator: 4,
                denominator: NoteValue::Sixteenth,
            },
            note_trigger_mode: NoteTriggerMode::Auto,
            trigger_threshold: Ratio::new::<ratio>(0.5),
            phase_offset: Ratio::zero(),

            shape_name: Some("Pyramid".to_owned()),
            shape_path: Some("factory/Classic/Pyramid.lfo".to_owned()),
            shape: vec![
                CurvePoint {
                    mode: CurvePointMode::Sharp,
                    x: 0.0,
                    y: -1.0,
                    curve_x: 1.0,
                    curve_y: 1.0,
                },
                CurvePoint {
                    mode: CurvePointMode::Sharp,
                    x: 0.5,
                    y: 1.0,
                    curve_x: 1.0,
                    curve_y: 1.0,
                },
            ],
            shape_edited: false,
            // Phase Plant 2.0 defaults to Pyramid. Phase Plant 1.0 defaulted
            // to Sine.
            // shape_name: Some("Sine".to_owned()),
            // shape_path: Some("factory/Classic/Sine.lfo".to_owned()),
            // shape: vec![
            //     CurvePoint::new_sharp(0.25, 1.0, 0.0, 0.0),
            //     CurvePoint::new_sharp(0.75, -1.0, 0.0, 0.0),
            // ],
        }
    }
}

impl Modulator for LfoModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Lfo
    }
}

impl dyn Modulator {
    #[must_use]
    pub fn as_lfo(&self) -> Option<&LfoModulator> {
        self.downcast_ref::<LfoModulator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::frequency::hertz;
    use uom::si::ratio::percent;

    use crate::generator::LoopMode;
    use crate::modulator::{GROUP_ID_NONE, OutputRange};
    use crate::point::CurvePoint;
    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn default() {
        let modulator = LfoModulator::default();
        assert_eq!(modulator.output_range, OutputRange::Unipolar);
        assert_eq!(modulator.depth.get::<percent>(), 100.0);
        assert_eq!(modulator.loop_mode, LoopMode::Infinite);
        assert!(!modulator.rate.sync);
        assert_eq!(modulator.rate.frequency.get::<hertz>(), 1.0);
        assert_eq!(modulator.rate.numerator, 4);
        assert_eq!(modulator.rate.denominator, NoteValue::Sixteenth);
        assert_relative_eq!(modulator.phase_offset.get::<ratio>(), 0.0);
        assert_eq!(modulator.trigger_threshold.get::<percent>(), 50.0);
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Auto);

        assert_eq!(modulator.shape_name, Some("Pyramid".to_owned()));
        assert_eq!(
            modulator.shape_path,
            Some("factory/Classic/Pyramid.lfo".to_owned())
        );
        assert_eq!(
            CurvePoint::new_sharp(0.0, -1.0, 1.0, 1.0),
            modulator.shape[0]
        );
        assert_eq!(
            CurvePoint::new_sharp(0.5, 1.0, 1.0, 1.0),
            modulator.shape[1]
        );
        assert!(!modulator.shape_edited);
    }

    /// Version 1 LFO presets cannot be compared against `LfoModulator::default()`
    /// because the default output range changed from `Bipolar` in Phase Plant 1
    /// to `Unipolar` in Phase Plant 2.
    #[test]
    fn init_version_1() {
        for file in &["lfo-1.7.7.phaseplant", "lfo-1.8.13.phaseplant"] {
            let preset = read_modulator_preset("lfo", file).unwrap();
            let container = preset.modulator_container(0).unwrap();
            assert_eq!(container.id, 0);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &LfoModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator.output_range, OutputRange::Bipolar);
            assert_eq!(modulator.depth.get::<percent>(), 100.0);
            assert_eq!(modulator.loop_mode, LoopMode::Infinite);
            assert!(!modulator.rate.sync);
            assert_eq!(modulator.rate.frequency.get::<hertz>(), 1.0);
            assert_eq!(modulator.rate.numerator, 4);
            assert_eq!(modulator.rate.denominator, NoteValue::Sixteenth);
            assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Auto);
            assert_eq!(modulator.trigger_threshold.get::<ratio>(), 0.5);
            assert_relative_eq!(modulator.phase_offset.get::<ratio>(), 0.0);

            assert_eq!(modulator.shape_name, Some("Sine".to_owned()));

            if preset
                .format_version
                .is_at_least(&PhasePlantRelease::V1_8_0.format_version())
            {
                assert_eq!(
                    modulator.shape_path,
                    Some("factory/Classic/Sine.lfo".to_owned())
                );
            }

            assert_eq!(
                CurvePoint::new_smooth(0.25, 1.0, 0.0, 0.0),
                modulator.shape[0]
            );
            assert_eq!(
                CurvePoint::new_smooth(0.75, -1.0, 0.0, 0.0),
                modulator.shape[1]
            );
            assert!(!modulator.shape_edited);
        }
    }

    #[test]
    fn init_version_2() {
        for file in &["lfo-2.0.14.phaseplant", "lfo-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("lfo", file).unwrap();
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &LfoModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &LfoModulator::default());
        }
    }

    #[test]
    fn inverted_42hz() {
        let preset = read_modulator_preset("lfo", "lfo-inverted-42hz-1.8.17.phaseplant").unwrap();
        assert_eq!(preset.modulator_containers.len(), 1);
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert!(!modulator.rate.sync);
        assert_relative_eq!(
            modulator.rate.frequency.get::<hertz>(),
            42.0,
            epsilon = 0.0001
        );
        assert_eq!(modulator.output_range, OutputRange::Inverted);
    }

    #[test]
    fn loop_mode_trigger_threshold() {
        let preset =
            read_modulator_preset("lfo", "lfo-ping_pong-trigger15-2.1.0.phaseplant").unwrap();
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.loop_mode, LoopMode::PingPong);
        assert_relative_eq!(
            modulator.trigger_threshold.get::<ratio>(),
            0.15,
            epsilon = 0.01
        );
    }

    /// Primarily testing the note trigger parameter.
    #[test]
    fn note_trigger() {
        let preset =
            read_modulator_preset("lfo", "lfo-note_trigger_legato-sync-2.1.0.phaseplant").unwrap();
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Legato);
        assert!(modulator.rate.sync);

        let preset =
            read_modulator_preset("lfo", "lfo-note_trigger_never-bipolar-2.1.0.phaseplant")
                .unwrap();
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Never);
        assert_eq!(modulator.output_range, OutputRange::Bipolar);

        let preset =
            read_modulator_preset("lfo", "lfo-note_trigger_note_on-inverted-2.1.0.phaseplant")
                .unwrap();
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Always);
        assert_eq!(modulator.output_range, OutputRange::Inverted);
    }

    #[test]
    fn oneshot_minimized() {
        let preset = read_modulator_preset("lfo", "lfo-1shot-minimized-1.8.13.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(container.minimized);
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.loop_mode, LoopMode::Off);
    }

    #[test]
    fn retrigger_oneshot_depth() {
        let preset =
            read_modulator_preset("lfo", "lfo-retrig_off-1shot-depth50-1.8.14.phaseplant").unwrap();
        assert_eq!(preset.modulator_containers.len(), 1);
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.loop_mode, LoopMode::Off);
        assert_relative_eq!(modulator.depth.get::<percent>(), 50.0);
    }

    /// Started out as a sine but both points were changed from smooth to sharp.
    #[test]
    fn sharp() {
        let preset = read_modulator_preset("lfo", "lfo-sharp-2.0.14.phaseplant").unwrap();
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.shape_name, Some("Sine".to_owned()));
        assert_eq!(
            modulator.shape_path,
            Some("factory/Classic/Sine.lfo".to_owned())
        );
        assert_eq!(
            CurvePoint::new_sharp(0.25, 1.0, 1.0, 1.0),
            modulator.shape[0]
        );
        assert_eq!(
            CurvePoint::new_sharp(0.75, -1.0, 1.0, 1.0),
            modulator.shape[1]
        );
        assert!(modulator.shape_edited);
    }

    #[test]
    fn square_sync_phase() {
        let preset =
            read_modulator_preset("lfo", "lfo-square-sync-phase25-1.8.14.phaseplant").unwrap();
        assert_eq!(preset.modulator_containers.len(), 1);
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert!(modulator.rate.sync);
        assert_relative_eq!(modulator.phase_offset.get::<ratio>(), 25.0 / 360.0);
        assert_eq!(modulator.shape_name, Some("Square".to_string()));
        assert_eq!(
            modulator.shape_path,
            Some("factory/Classic/Square.lfo".to_string())
        );
    }

    /// An LFO at 5Hz instead of 1Hz and with a sustain loop mode.
    #[test]
    fn sustain_5hz() {
        let preset = read_modulator_preset("lfo", "lfo-sustain-5hz-2.0.14.phaseplant").unwrap();
        let modulator: &LfoModulator = preset.modulator(0).unwrap();
        assert!(!modulator.rate.sync);
        assert_eq!(modulator.loop_mode, LoopMode::Sustain);
        assert_relative_eq!(
            modulator.rate.frequency.get::<hertz>(),
            5.0,
            epsilon = 0.0001
        );

        // The shape doesn't change based on the rate.c:
        assert_eq!(
            CurvePoint::new_sharp(0.0, -1.0, 1.0, 1.0),
            modulator.shape[0]
        );
        assert_eq!(
            CurvePoint::new_sharp(0.5, 1.0, 1.0, 1.0),
            modulator.shape[1]
        );
    }
}

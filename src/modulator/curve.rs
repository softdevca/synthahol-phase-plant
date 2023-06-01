//! [Curve Modulator](https://kilohearts.com/docs/modulation#curve)

// As of Phase Plant 2.1.0 the pan and zoom settings of the curve editor are
// not saved.

use std::any::Any;

use uom::si::f32::Frequency;
use uom::si::frequency::hertz;
use uom::si::ratio::ratio;

use crate::generator::LoopMode;
use crate::modulator::{Modulator, ModulatorMode, NoteTriggerMode, OutputRange};
use crate::point::{CurvePoint, CurvePointMode};
use crate::*;

#[derive(Debug, PartialEq)]
pub struct CurveModulator {
    pub output_range: OutputRange,
    pub loop_mode: LoopMode,
    pub note_trigger_mode: NoteTriggerMode,

    /// In the Phase Plant interface the rate is specified as time. It is
    /// converted to the rate frequency on save and load to avoid duplication.
    pub rate: Rate,

    /// Ranges from -1.0..=1.0
    pub trigger_threshold: f32,

    pub depth: Ratio,

    pub shape: Vec<CurvePoint>,
    pub shape_name: Option<String>,
    pub shape_path: Option<String>,
    pub shape_edited: bool,
}

impl Default for CurveModulator {
    fn default() -> Self {
        let modulator = Self {
            output_range: OutputRange::Unipolar,
            loop_mode: LoopMode::Off,
            rate: Rate {
                sync: false,
                frequency: Frequency::new::<hertz>(1.0),
                numerator: 4,
                denominator: NoteValue::Sixteenth,
            },
            note_trigger_mode: NoteTriggerMode::Auto,
            trigger_threshold: 0.5,
            depth: Ratio::new::<ratio>(1.0),
            shape_name: Some("Slope".to_owned()),
            shape_path: Some("factory/Simple/Slope.lfo".to_owned()),
            shape: vec![
                CurvePoint {
                    mode: CurvePointMode::Smooth,
                    x: 0.0,
                    y: 1.0,
                    curve_x: 0.0,
                    curve_y: 0.0,
                },
                CurvePoint {
                    mode: CurvePointMode::Smooth,
                    x: 1.0,
                    y: -1.0,
                    curve_x: 0.0,
                    curve_y: 0.0,
                },
            ],
            shape_edited: false,
        };
        modulator
    }
}

impl Modulator for CurveModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Curve
    }
}

impl dyn Modulator {
    #[must_use]
    pub fn as_curve(&self) -> Option<&CurveModulator> {
        self.downcast_ref::<CurveModulator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::frequency::hertz;
    use uom::si::ratio::percent;

    use crate::generator::LoopMode;
    use crate::modulator::OutputRange;
    use crate::point::CurvePoint;
    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn default() {
        let modulator = CurveModulator::default();
        assert_eq!(modulator.output_range, OutputRange::Unipolar);
        assert_eq!(modulator.loop_mode, LoopMode::Off);
        assert!(!modulator.rate.sync);
        assert_eq!(modulator.rate.frequency.get::<hertz>(), 1.0);
        assert_eq!(modulator.rate.numerator, 4);
        assert_eq!(modulator.rate.denominator, NoteValue::Sixteenth);
        assert_eq!(modulator.depth.get::<percent>(), 100.0);
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Auto);

        assert_eq!(modulator.shape_name, Some("Slope".to_owned()));
        assert_eq!(
            modulator.shape_path,
            Some("factory/Simple/Slope.lfo".to_owned())
        );
        assert_eq!(
            modulator.shape[0],
            CurvePoint::new_smooth(0.0, 1.0, 0.0, 0.0),
        );
        assert_eq!(
            modulator.shape[1],
            CurvePoint::new_smooth(1.0, -1.0, 0.0, 0.0)
        );
        assert!(!modulator.shape_edited);
    }

    #[test]
    fn init() {
        for file in &["curve-2.0.12.phaseplant", "curve-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("curve", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &CurveModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &CurveModulator::default());
        }
    }

    #[test]
    fn loop_mode() {
        let preset = read_modulator_preset("curve", "curve-ping_pong-2.0.12.phaseplant").unwrap();
        let modulator: &CurveModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.loop_mode, LoopMode::PingPong);
    }

    #[test]
    fn point_appended() {
        let preset =
            read_modulator_preset("curve", "curve-point_appended-2.1.0.phaseplant").unwrap();
        let modulator: &CurveModulator = preset.modulator(0).unwrap();
        assert!(modulator.shape_edited);
        assert_eq!(modulator.shape.len(), 3);
        assert!(modulator.shape[0].is_smooth());
        assert_relative_eq!(modulator.shape[2].x, 1.1824, epsilon = 0.001);
        assert_relative_eq!(modulator.shape[2].y, 0.0516, epsilon = 0.001);
        assert_relative_eq!(modulator.shape[2].curve_x, 0.0, epsilon = 0.001);
        assert_relative_eq!(modulator.shape[2].curve_y, 1.0, epsilon = 0.001);
    }

    #[test]
    fn point_mode() {
        let preset = read_modulator_preset("curve", "curve-points_sharp-2.1.0.phaseplant").unwrap();
        let modulator: &CurveModulator = preset.modulator(0).unwrap();
        assert!(modulator.shape_edited);
        assert_eq!(modulator.shape.len(), 2);
        assert!(modulator.shape[0].is_sharp());
        assert!(modulator.shape[1].is_sharp());

        let preset =
            read_modulator_preset("curve", "curve-points_smooth-2.1.0.phaseplant").unwrap();
        let modulator: &CurveModulator = preset.modulator(0).unwrap();
        assert!(modulator.shape_edited);
        assert_eq!(modulator.shape.len(), 2);
        assert!(modulator.shape[0].is_smooth());
        assert!(modulator.shape[1].is_smooth());
    }
    #[test]
    fn rate() {
        let preset = read_modulator_preset("curve", "curve-rate50ms-2.0.16.phaseplant").unwrap();
        let modulator: &CurveModulator = preset.modulator(0).unwrap();
        assert!(!modulator.rate.sync);
        assert_relative_eq!(
            modulator.rate.frequency.get::<hertz>(),
            20.0,
            epsilon = 0.001
        );

        let preset = read_modulator_preset("curve", "curve-rate532-2.0.16.phaseplant").unwrap();
        let modulator: &CurveModulator = preset.modulator(0).unwrap();
        assert!(modulator.rate.sync);
        assert_eq!(modulator.rate.numerator, 5);
        assert_eq!(modulator.rate.denominator, NoteValue::ThirtySecond);
    }
}

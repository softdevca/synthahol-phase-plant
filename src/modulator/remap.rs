//! [Remap Modulator](https://kilohearts.com/docs/modulation#remap)

use std::any::Any;

use crate::modulator::{Modulator, ModulatorMode};

use super::*;

#[derive(Debug, PartialEq)]
pub struct RemapModulator {
    pub depth: Ratio,

    /// [`OutputRange::Inverted`] is not a legal option so the full `OutputRange`
    /// enumeration is not used.
    pub bipolar: bool,

    pub shape: Vec<CurvePoint>,
    pub shape_name: Option<String>,
    pub shape_path: Option<String>,
    pub shape_edited: bool,
}

impl Modulator for RemapModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Remap
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn curve() {
        let preset = read_modulator_preset("remap", "remap-curve-2.1.0.phaseplant").unwrap();
        let modulator: &RemapModulator = preset.modulator(0).unwrap();
        assert!(modulator.shape_edited);
        assert_eq!(modulator.shape_name, Some("Linear".to_owned()));
        assert_eq!(
            modulator.shape_path,
            Some("factory/Simple/Linear.remap".to_owned())
        );
        assert_eq!(modulator.shape.len(), 3);
        assert_eq!(
            modulator.shape[0],
            CurvePoint::new_sharp(-1.0, 0.15833092, 2.0, 0.0)
        );
        assert_eq!(
            modulator.shape[1],
            CurvePoint::new_smooth(-0.09177613, 0.7327706, 0.0, 2.0)
        );
        assert_eq!(
            modulator.shape[2],
            CurvePoint::new_sharp(1.0, -0.28473783, 1.0, 1.0)
        );
    }

    #[test]
    fn init() {
        for file in &[
            "remap-2.0.0.phaseplant",
            "remap-2.0.12.phaseplant",
            "remap-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("remap", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &RemapModulator = preset.modulator(0).unwrap();
            assert!(modulator.bipolar);
            assert_relative_eq!((modulator.depth.get::<percent>()), 100.0);
            assert_eq!(modulator.shape_name, Some("Linear".to_owned()));
            assert_eq!(
                modulator.shape_path,
                Some("factory/Simple/Linear.remap".to_owned())
            );
            assert_eq!(modulator.shape.len(), 2);
            assert_eq!(
                modulator.shape[0],
                CurvePoint::new_sharp(-1.0, -1.0, 1.0, 1.0)
            );
            assert_eq!(
                modulator.shape[1],
                CurvePoint::new_sharp(1.0, 1.0, 1.0, 1.0)
            );
        }
    }
}

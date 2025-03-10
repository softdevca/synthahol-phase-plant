//! [Velocity Modulator](https://kilohearts.com/docs/modulation#velocity)

use std::any::Any;

use strum_macros::{Display, FromRepr};
use uom::si::ratio::ratio;

use super::*;

#[derive(Clone, Copy, Debug, Display, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum VelocityTriggerMode {
    // The discriminants correspond to the file format.
    Strike = 0,
    Release = 1,
    Both = 2,
}

impl VelocityTriggerMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown trigger mode {id}")))
    }
}

#[derive(Debug, PartialEq)]
pub struct VelocityModulator {
    pub output_range: OutputRange,
    pub depth: Ratio,
    pub trigger_mode: VelocityTriggerMode,
}

impl Default for VelocityModulator {
    fn default() -> Self {
        Self {
            output_range: OutputRange::Unipolar,
            depth: Ratio::new::<ratio>(1.0),
            trigger_mode: VelocityTriggerMode::Strike,
        }
    }
}

impl Modulator for VelocityModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Velocity
    }
}

#[cfg(test)]
mod test {
    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &[
            "velocity-1.7.0.phaseplant",
            "velocity-1.7.11.phaseplant",
            "velocity-1.8.0.phaseplant",
            "velocity-1.8.13.phaseplant",
            "velocity-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("velocity", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            let modulator: &VelocityModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }

    #[test]
    fn parts() {
        let preset =
            read_modulator_preset("velocity", "velocity-both-bipolar-2.1.0.phaseplant").unwrap();
        let modulator: &VelocityModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.output_range, OutputRange::Bipolar);
        assert_eq!(modulator.trigger_mode, VelocityTriggerMode::Both);

        let preset =
            read_modulator_preset("velocity", "velocity-release-depth50-2.1.0.phaseplant").unwrap();
        let modulator: &VelocityModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.trigger_mode, VelocityTriggerMode::Release);
        assert_eq!(modulator.depth.get::<ratio>(), 0.5);
    }
}

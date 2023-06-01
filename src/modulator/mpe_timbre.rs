//! [MPE Timbre Modulator](https://kilohearts.com/docs/modulation#mpe_timbre)

use std::any::Any;

use uom::si::ratio::ratio;

use super::*;

#[derive(Debug, PartialEq)]
pub struct MpeTimbreModulator {
    pub output_range: OutputRange,
    pub depth: Ratio,
}

impl Default for MpeTimbreModulator {
    fn default() -> Self {
        Self {
            output_range: OutputRange::Bipolar,
            depth: Ratio::new::<ratio>(1.0),
        }
    }
}

impl Modulator for MpeTimbreModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::MpeTimbre
    }
}

#[cfg(test)]
mod test {
    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &[
            "mpe_timbre-2.0.12.phaseplant",
            "mpe_timbre-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("mpe_timbre", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            let modulator: &MpeTimbreModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }
}

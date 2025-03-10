//! [Slew Limiter Modulator](https://kilohearts.com/docs/modulation#slew_limiter)
//!
//! The Slew Limiter modulator was added in Phase Plant 2.0.13.

use std::any::Any;

use uom::si::time::millisecond;

use super::*;

#[derive(Debug, PartialEq)]
pub struct SlewLimiterModulator {
    pub attack: Time,
    pub decay: Time,
    pub linked: bool,
}

impl Default for SlewLimiterModulator {
    fn default() -> Self {
        Self {
            attack: Time::new::<millisecond>(100.0),
            decay: Time::new::<millisecond>(100.0),
            linked: true,
        }
    }
}

impl Modulator for SlewLimiterModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::SlewLimiter
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::time::second;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn default() {
        let modulator = SlewLimiterModulator::default();
        assert_relative_eq!(modulator.attack.get::<second>(), 0.1);
        assert_relative_eq!(modulator.decay.get::<second>(), 0.1);
        assert!(modulator.linked);
    }

    #[test]
    fn init() {
        for file in &[
            "slew_limiter-2.0.13.phaseplant",
            "slew_limiter-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("slew_limiter", file).unwrap();
            let modulator: &SlewLimiterModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "slew_limiter",
            "slew_limiter-att200-dec300-unlinked-2.1.0.phaseplant",
        )
        .unwrap();
        let modulator: &SlewLimiterModulator = preset.modulator(0).unwrap();
        assert!(!modulator.linked);
        assert_relative_eq!(modulator.attack.get::<second>(), 0.2);
        assert_relative_eq!(modulator.decay.get::<second>(), 0.3);
    }
}

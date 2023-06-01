//! [Envelope Modulator](https://kilohearts.com/docs/modulation#envelope)

use std::any::Any;

use uom::si::ratio::percent;

use super::*;

#[derive(Debug, PartialEq)]
pub struct EnvelopeModulator {
    pub envelope: Envelope,
    pub depth: Ratio,
}

impl Default for EnvelopeModulator {
    fn default() -> Self {
        Self {
            envelope: Default::default(),
            depth: Ratio::new::<percent>(100.0),
        }
    }
}

impl Modulator for EnvelopeModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Envelope
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;
    use uom::si::time::second;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &["envelope-1.8.13.phaseplant", "envelope-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("envelope", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert_eq!(container.id, 0);
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &EnvelopeModulator = preset.modulator(0).unwrap();
            assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
            let envelope = &modulator.envelope;
            assert_relative_eq!(envelope.delay.get::<second>(), 0.0);
            assert_relative_eq!(envelope.attack.get::<second>(), 0.010, epsilon = 0.00001);
            assert_relative_eq!(envelope.attack_curve, 0.0);
            assert_relative_eq!(envelope.hold.get::<second>(), 0.0);
            assert_relative_eq!(envelope.decay.get::<second>(), 0.100, epsilon = 0.0001);
            assert_relative_eq!(envelope.decay_falloff, 0.0);
            assert_relative_eq!(envelope.sustain.get::<percent>(), 1.0);
            assert_relative_eq!(envelope.release.get::<second>(), 0.100, epsilon = 0.0001);
            assert_relative_eq!(envelope.release_falloff, 0.0);
        }
    }

    #[test]
    fn eleven_to_sixteen() {
        let preset =
            read_modulator_preset("envelope", "envelope-11to16-1.8.13.phaseplant").unwrap();
        let modulator: &EnvelopeModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
        let envelope = &modulator.envelope;
        assert_relative_eq!(envelope.delay.get::<second>(), 0.011, epsilon = 0.0001);
        assert_relative_eq!(envelope.attack.get::<second>(), 0.012, epsilon = 0.0001);
        assert_relative_eq!(envelope.attack_curve, 0.0);
        assert_relative_eq!(envelope.hold.get::<second>(), 0.013, epsilon = 0.0001);
        assert_relative_eq!(envelope.decay.get::<second>(), 0.014, epsilon = 0.0001);
        assert_relative_eq!(envelope.decay_falloff, 0.0);
        assert_relative_eq!(envelope.sustain.get::<percent>(), 0.15, epsilon = 0.0001);
        assert_relative_eq!(envelope.release.get::<second>(), 0.016, epsilon = 0.0001);
        assert_relative_eq!(envelope.release_falloff, 0.0);
    }

    #[test]
    fn curves() {
        let preset =
            read_modulator_preset("envelope", "envelope-curves25-50-75-1.8.13.phaseplant").unwrap();
        let modulator: &EnvelopeModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.envelope.attack_curve, 0.25);
        assert_relative_eq!(modulator.envelope.decay_falloff, 0.50);
        assert_relative_eq!(modulator.envelope.release_falloff, 0.75);
    }

    #[test]
    fn disabled() {
        let preset =
            read_modulator_preset("envelope", "envelope-disabled-1.8.13.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(!container.enabled);
    }

    #[test]
    fn depth() {
        let preset =
            read_modulator_preset("envelope", "envelope-minimized-depth50-1.8.13.phaseplant")
                .unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(container.minimized);
        let modulator: &EnvelopeModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.depth.get::<percent>(), 50.0);
    }
}

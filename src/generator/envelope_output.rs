//! The [EnvelopeOutput](https://kilohearts.com/docs/phase_plant/#envelope_output)
//! generator controls the level of the output with an envelope.

use std::any::Any;

use uom::si::f32::Time;
use uom::si::time::second;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct EnvelopeOutput {
    pub id: GeneratorId,
    pub enabled: bool,
    pub output_enabled: bool,
    pub name: String,
    pub gain: Decibels,
    pub pan: Ratio,
    pub destination: OutputDestination,
    pub envelope: Envelope,
}

impl Default for EnvelopeOutput {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::EnvelopeOutput.name().to_owned(),
            output_destination: OutputDestination::Lane1,
            envelope: Envelope {
                delay: Time::zero(),
                attack: Time::new::<second>(0.001),
                attack_curve: 0.5,
                hold: Time::zero(),
                decay: Time::new::<second>(0.1),
                decay_falloff: 0.75,
                sustain: Ratio::zero(),
                release: Time::new::<second>(0.005),
                release_falloff: 0.75,
            },
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for EnvelopeOutput {
    fn from(block: &GeneratorBlock) -> Self {
        EnvelopeOutput {
            id: block.id,
            enabled: block.enabled,
            output_enabled: block.output_enabled,
            name: block.name.to_owned(),
            gain: block.output_gain,
            pan: block.pan,
            destination: block.output_destination,
            envelope: block.envelope.clone(),
        }
    }
}

impl Generator for EnvelopeOutput {
    fn id(&self) -> Option<GeneratorId> {
        Some(self.id)
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::EnvelopeOutput
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_envelope_output(&self) -> Option<&EnvelopeOutput> {
        self.downcast_ref::<EnvelopeOutput>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;
    use uom::si::time::second;

    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn disabled() {
        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-disabled-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn init() {
        for file in &[
            "envelope_output-1.7.0.phaseplant",
            "envelope_output-1.8.13.phaseplant",
            "envelope_output-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("envelope_output", file).unwrap();
            let generator: &EnvelopeOutput = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert!(generator.output_enabled);
            assert_eq!(generator.name(), "Envelope".to_owned());
            assert_eq!(generator.destination, OutputDestination::Lane1);
            assert_relative_eq!(generator.gain.db(), -12.04, epsilon = 0.01);
            assert_relative_eq!(generator.pan.get::<percent>(), 0.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-attack_curve25-hold50-lane3-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert_eq!(generator.destination, OutputDestination::Lane3);
        assert_relative_eq!(generator.envelope.attack_curve, 0.25, epsilon = 0.0001);
        assert_relative_eq!(
            generator.envelope.hold.get::<second>(),
            0.05,
            epsilon = 0.0001
        );

        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-decay50-decay_curve25-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert_eq!(generator.destination, OutputDestination::Lane1);
        assert_relative_eq!(
            generator.envelope.decay.get::<second>(),
            0.05,
            epsilon = 0.0001
        );
        assert_relative_eq!(generator.envelope.decay_falloff, 0.25, epsilon = 0.0001);

        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-delay100-attack200-lane2-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert!(generator.enabled);
        assert_eq!(generator.destination, OutputDestination::Lane2);
        assert_relative_eq!(
            generator.envelope.attack.get::<second>(),
            0.2,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            generator.envelope.delay.get::<second>(),
            0.1,
            epsilon = 0.0001
        );

        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-gain-20-pan50-sideband-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert_eq!(generator.destination, OutputDestination::Sideband);
        assert_relative_eq!(generator.gain.db(), -20.0, epsilon = 0.0001);
        assert_relative_eq!(generator.pan.get::<percent>(), 50.0, epsilon = 0.0001);

        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-sus50-rel25-rel_curve5-none-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert_eq!(generator.destination, OutputDestination::None);
        assert_relative_eq!(
            generator.envelope.sustain.get::<percent>(),
            0.5,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            generator.envelope.release.get::<second>(),
            0.025,
            epsilon = 0.0001
        );
        assert_relative_eq!(generator.envelope.release_falloff, 0.05, epsilon = 0.0001);
    }

    #[test]
    fn parts_version_2() {
        let preset = read_generator_preset(
            "envelope_output",
            "envelope_output-out_disabled-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &EnvelopeOutput = preset.generator(1).unwrap();
        assert!(!generator.output_enabled);
    }
}

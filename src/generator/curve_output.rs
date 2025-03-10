//! The [Curve Output](https://kilohearts.com/docs/phase_plant/#curve_output)
//! generator controls the level of the output with an editable curve.
//!
//! Curve Output was added to Phase Plant version 2.0

use std::any::Any;

use crate::point::CurvePoint;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CurveOutput {
    pub id: GeneratorId,
    pub enabled: bool,
    pub output_enabled: bool,
    pub name: String,
    pub gain: Decibels,
    pub pan: Ratio,
    pub destination: OutputDestination,
    pub loop_mode: LoopMode,
    pub loop_start: Ratio,
    pub loop_length: Ratio,
    pub rate: Rate,
    pub settings_locked: bool,
    pub curve: Vec<CurvePoint>,
    pub curve_edited: bool,
    pub curve_length: Time,
    pub curve_name: Option<String>,
    pub curve_path: Option<String>,
}

impl Default for CurveOutput {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::CurveOutput.name().to_owned(),
            output_destination: OutputDestination::Lane1,
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for CurveOutput {
    fn from(block: &GeneratorBlock) -> Self {
        Self {
            id: block.id,
            enabled: block.enabled,
            output_enabled: block.output_enabled,
            name: block.name.to_owned(),
            gain: block.output_gain,
            pan: block.pan,
            rate: block.rate.clone(),
            destination: block.output_destination,
            loop_mode: block.curve_loop_mode,
            loop_start: block.curve_loop_start,
            loop_length: block.curve_loop_length,
            settings_locked: block.settings_locked,
            curve: block.curve.clone(),
            curve_edited: block.curve_edited,
            curve_length: block.curve_length,
            curve_name: block.curve_name.clone(),
            curve_path: block.curve_path.clone(),
        }
    }
}

impl Generator for CurveOutput {
    fn id(&self) -> Option<GeneratorId> {
        Some(self.id)
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::CurveOutput
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_curve_output(&self) -> Option<&CurveOutput> {
        self.downcast_ref::<CurveOutput>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;
    use uom::si::time::{millisecond, second};

    use crate::test::read_generator_preset;

    use super::*;

    /// The default configuration is with the Slope curve. This tests the
    /// default configuration with all points removed from the curve.
    #[test]
    fn blank() {
        let preset =
            read_generator_preset("curve_output", "curve_output-blank-2.1.0.phaseplant").unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert!(generator.curve.is_empty());
        assert!(generator.curve_name.is_none());
        assert!(generator.curve_path.is_none());
    }

    /// Bounced is a factory curve.
    #[test]
    fn bounced() {
        for file in &[
            "curve_output-bounced-2.0.12.phaseplant",
            "curve_output-bounced-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("curve_output", file).unwrap();
            let generator: &CurveOutput = preset.generator(1).unwrap();
            assert_eq!(generator.destination, OutputDestination::Lane1);
            assert_eq!(generator.curve_name, Some("Bounced".to_owned()));
            assert_eq!(
                generator.curve_path,
                Some("factory/Complex/Bounced.curve".to_owned())
            );
            assert_eq!(generator.curve.len(), 49);

            let first = generator.curve.first().unwrap();
            assert!(first.is_sharp());

            let not_first = generator.curve.get(1).unwrap();
            assert!(!not_first.is_sharp());
        }
    }

    #[test]
    fn init() {
        for file in &[
            "curve_output-2.0.0.phaseplant",
            "curve_output-2.0.12.phaseplant",
            "curve_output-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("curve_output", file).unwrap();
            let generator: &CurveOutput = preset.generator(1).unwrap();
            assert!(generator.enabled);
            assert!(!generator.settings_locked);
            assert_eq!(generator.loop_mode, LoopMode::Off);
            assert_eq!(generator.loop_start.get::<percent>(), 0.0);
            assert_eq!(generator.loop_length.get::<percent>(), 100.0);
            assert_eq!(generator.name(), "Curve".to_owned());
            assert_eq!(generator.destination, OutputDestination::Lane1);
            assert_relative_eq!(generator.gain.db(), -12.04, epsilon = 0.01);
            assert_relative_eq!(generator.pan.get::<percent>(), 0.0);

            assert!(!generator.curve_edited);
            assert_relative_eq!(generator.curve_length.get::<second>(), 1.0);
            assert_eq!(generator.curve_name, Some("Slope".to_owned()));
            assert!(generator.curve_path.is_none());
            // TODO: Check rest of curve
        }
    }

    #[test]
    fn loop_start_and_length() {
        let preset = read_generator_preset(
            "curve_output",
            "curve_output-loop_start25-loop_length50-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert_eq!(generator.loop_mode, LoopMode::Sustain);
        assert_eq!(generator.loop_start.get::<percent>(), 25.0);
        assert_eq!(generator.loop_length.get::<percent>(), 50.0);
    }

    #[test]
    fn parts() {
        let preset = read_generator_preset(
            "curve_output",
            "curve_output-5sec-settings_locked-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert!(generator.settings_locked);
        assert_relative_eq!(
            generator.curve_length.get::<second>(),
            5.0,
            epsilon = 0.0001
        );

        let preset =
            read_generator_preset("curve_output", "curve_output-disabled-2.1.0.phaseplant")
                .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert!(!generator.enabled);

        let preset = read_generator_preset(
            "curve_output",
            "curve_output-gain3-pan25-lane2-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert_eq!(generator.destination, OutputDestination::Lane2);
        assert_relative_eq!(generator.gain.db(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(generator.pan.get::<percent>(), 25.0);

        let preset = read_generator_preset(
            "curve_output",
            "curve_output-sustain-length10ms-2.0.12.phaseplant",
        )
        .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert_eq!(generator.loop_mode, LoopMode::Sustain);
        assert_relative_eq!(generator.curve_length.get::<millisecond>(), 10.0);

        let preset =
            read_generator_preset("curve_output", "curve_output-sync-reverse-2.1.0.phaseplant")
                .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert!(generator.rate.sync);
        assert_eq!(generator.loop_mode, LoopMode::Reverse);
    }

    /// The default Slope shape with the first point set to 50% at 0 ms and
    /// the second at 25% at 500 ms.
    #[test]
    fn slope_edited() {
        let preset = read_generator_preset(
            "curve_output",
            "curve_output-0ms,50-500ms,25-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &CurveOutput = preset.generator(1).unwrap();
        assert!(generator.curve_edited);
    }
}

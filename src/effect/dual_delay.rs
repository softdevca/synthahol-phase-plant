//! [Dual Delay](https://kilohearts.com/products/dual_delay) is a cross-feeding
//! echo effect.
//!
//! Dual Delay was added to Phase Plant in version 2.0.9.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 2.0.12              | 1012           |
//! | 2.0.16              | 1013           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::{percent, ratio};
use uom::si::time::{millisecond, second};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct DualDelay {
    pub time: Time,
    pub second_delay_length: Ratio,
    pub sync: bool,
    pub tone: Ratio,
    pub feedback: Ratio,
    pub spread: Ratio,
    pub duck: Ratio,
    pub crosstalk: Ratio,
    pub mix: Ratio,
}

impl Default for DualDelay {
    fn default() -> Self {
        Self {
            time: Time::new::<millisecond>(200.0),
            second_delay_length: Ratio::new::<ratio>(1.618034),
            sync: false,
            tone: Ratio::zero(),
            feedback: Ratio::new::<percent>(50.0),
            spread: Ratio::new::<percent>(50.0),
            duck: Ratio::zero(),
            crosstalk: Ratio::new::<percent>(50.0),
            mix: Ratio::new::<ratio>(1.0 / 3.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_dual_delay(&self) -> Option<&DualDelay> {
        self.downcast_ref::<DualDelay>()
    }
}

impl Effect for DualDelay {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::DualDelay
    }
}

impl EffectRead for DualDelay {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1012 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let time = reader.read_seconds()?;
        let second_delay_length = reader.read_ratio()?;
        let feedback = reader.read_ratio()?;
        let crosstalk = reader.read_ratio()?;
        let spread = reader.read_ratio()?;
        let tone = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "dual_delay_unknown_3")?;
        reader.expect_u32(0, "dual_delay_unknown_4")?;
        reader.expect_u32(0, "dual_delay_unknown_5")?;
        reader.expect_u32(3, "dual_delay_unknown_6")?;
        reader.expect_u32(4, "dual_delay_unknown_7")?;

        let sync = reader.read_bool32()?;
        let duck = reader.read_ratio()?;

        Ok(EffectReadReturn::new(
            Box::new(DualDelay {
                time,
                second_delay_length,
                sync,
                tone,
                feedback,
                spread,
                duck,
                crosstalk,
                mix,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for DualDelay {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.time.get::<second>())?;
        writer.write_f32(self.second_delay_length.get::<ratio>())?;
        writer.write_f32(self.feedback.get::<ratio>())?;
        writer.write_f32(self.crosstalk.get::<ratio>())?;
        writer.write_f32(self.spread.get::<ratio>())?;
        writer.write_f32(self.tone.get::<ratio>())?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // dual_delay_unknown_3
        writer.write_u32(0)?; // dual_delay_unknown_4
        writer.write_u32(0)?; // dual_delay_unknown_5
        writer.write_u32(3)?; // dual_delay_unknown_6
        writer.write_u32(4)?; // dual_delay_unknown_7

        writer.write_bool32(self.sync)?;
        writer.write_f32(self.duck.get::<ratio>())?;

        writer.write_u32(0)?;
        Ok(())
    }

    fn write_version(&self) -> u32 {
        1013
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::time::{millisecond, second};

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = DualDelay::default();
        assert_eq!(effect.time.get::<second>(), 0.200);
        assert_relative_eq!(effect.second_delay_length.get::<ratio>(), 1.618034);
        assert!(!effect.sync);
        assert_relative_eq!(effect.tone.get::<percent>(), 0.0);
        assert_relative_eq!(effect.feedback.get::<percent>(), 50.0);
        assert_relative_eq!(effect.spread.get::<percent>(), 50.0);
        assert_relative_eq!(effect.duck.get::<percent>(), 0.0);
        assert_relative_eq!(effect.crosstalk.get::<percent>(), 50.0);
        assert_relative_eq!(effect.mix.get::<ratio>(), 1.0 / 3.0);
    }

    #[test]
    fn eq() {
        let effect = DualDelay::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, DualDelay::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "dual_delay-2.0.12.phaseplant",
            "dual_delay-2.0.16.phaseplant",
            "dual_delay-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("dual_delay", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_dual_delay().unwrap();
            assert_eq!(effect.time.get::<second>(), 0.200);
            assert_relative_eq!(effect.second_delay_length.get::<ratio>(), 1.618034);
            assert!(!effect.sync);
            assert_relative_eq!(effect.tone.get::<percent>(), 0.0);
            assert_relative_eq!(effect.feedback.get::<percent>(), 50.0);
            assert_relative_eq!(effect.spread.get::<percent>(), 50.0);
            assert_relative_eq!(effect.duck.get::<percent>(), 0.0);
            assert_relative_eq!(effect.crosstalk.get::<percent>(), 50.0);
            assert_relative_eq!(effect.mix.get::<ratio>(), 1.0 / 3.0);
        }
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset(
            "dual_delay",
            "dual_delay-1.25-sync-duck25-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(
            effect.second_delay_length.get::<ratio>(),
            1.25,
            epsilon = 0.00001
        );
        assert!(effect.sync);
        assert_eq!(effect.duck.get::<percent>(), 25.0);

        let preset =
            read_effect_preset("dual_delay", "dual_delay-903ms-duck25-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.duck.get::<percent>(), 24.57, epsilon = 0.01);
        assert_relative_eq!(effect.time.get::<millisecond>(), 903.0, epsilon = 0.1);

        let preset = read_effect_preset(
            "dual_delay",
            "dual_delay-crosstalk39-tone--35-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.crosstalk.get::<percent>(), 38.7, epsilon = 0.01);
        assert_relative_eq!(effect.tone.get::<percent>(), -34.5, epsilon = 0.1);

        let preset =
            read_effect_preset("dual_delay", "dual_delay-100ms-tone25-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.time.get::<millisecond>(), 100.0, epsilon = 0.001);
        assert_eq!(effect.tone.get::<percent>(), 25.0);

        let preset = read_effect_preset(
            "dual_delay",
            "dual_delay-disabled-reflectiverb-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.time.get::<millisecond>(), 71.0, epsilon = 0.1);
        assert_relative_eq!(
            effect.second_delay_length.get::<ratio>(),
            1.618034,
            epsilon = 0.00001
        );
        assert_relative_eq!(effect.tone.get::<percent>(), -46.3, epsilon = 0.1);
        assert_relative_eq!(effect.feedback.get::<percent>(), 80.9, epsilon = 0.1);
        assert_eq!(effect.spread.get::<percent>(), 25.0);
        assert_eq!(effect.duck.get::<percent>(), 0.0);
        assert_eq!(effect.crosstalk.get::<percent>(), 50.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 34.0, epsilon = 0.1);

        let preset = read_effect_preset(
            "dual_delay",
            "dual_delay-feedback25-spread75-crosstalk30-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_eq!(effect.feedback.get::<percent>(), 25.0);
        assert_eq!(effect.spread.get::<percent>(), 75.0);
        assert_relative_eq!(effect.crosstalk.get::<percent>(), 30.0, epsilon = 0.01);

        let preset = read_effect_preset(
            "dual_delay",
            "dual_delay-feedback70-spread55-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.feedback.get::<percent>(), 70.2, epsilon = 0.1);
        assert_relative_eq!(effect.spread.get::<percent>(), 54.9, epsilon = 0.1);

        let preset =
            read_effect_preset("dual_delay", "dual_delay-mix75-minimized-2.0.16.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_eq!(effect.mix.get::<percent>(), 75.0);

        let preset =
            read_effect_preset("dual_delay", "dual_delay-time-mix45-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.time.get::<millisecond>(), 200.0, epsilon = 0.001);
        assert_relative_eq!(effect.mix.get::<percent>(), 45.1, epsilon = 0.01);
        assert!(effect.sync);

        let preset =
            read_effect_preset("dual_delay", "dual_delay-x2-minimized-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_eq!(effect.second_delay_length.get::<ratio>(), 2.0);
    }

    #[test]
    fn tone() {
        let preset =
            read_effect_preset("dual_delay", "dual_delay-tone-25-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dual_delay().unwrap();
        assert_relative_eq!(effect.tone.get::<percent>(), -25.0, epsilon = 0.01);
    }
}

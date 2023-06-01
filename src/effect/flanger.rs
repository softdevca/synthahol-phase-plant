//! [Flanger](https://kilohearts.com/products/flanger) is an effect that
//! mixes a sound with a slightly delayed version of itself.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1002           |
//! | 1.8.14              | 1002           |
//! | 2.0.16              | 1013           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::{percent, ratio};
use uom::si::time::second;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Flanger {
    pub delay: Time,
    pub depth: Time,
    pub rate: f32,
    pub scroll: bool,

    /// Percentage of 360 degrees
    pub offset: Ratio,

    pub motion: f32,
    pub spread: f32,
    pub feedback: f32,
    pub mix: Ratio,
}

impl Flanger {
    pub fn offset_degrees(&self) -> f32 {
        self.offset.get::<percent>() * 360.0
    }
}

impl Default for Flanger {
    fn default() -> Self {
        Flanger {
            delay: Time::new::<second>(0.001),
            depth: Time::new::<second>(0.00103),
            rate: 0.31,
            scroll: true,
            offset: Ratio::zero(),
            motion: 0.5,
            spread: 0.25,
            feedback: 0.0,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_flanger(&self) -> Option<&Flanger> {
        self.downcast_ref::<Flanger>()
    }
}

impl Effect for Flanger {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Flanger
    }
}

impl EffectRead for Flanger {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1002 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let delay = Time::new::<second>(reader.read_f32()?);
        let depth = Time::new::<second>(reader.read_f32()?);
        let rate = reader.read_f32()?;

        let offset = reader.read_f32()?;
        if !(0.0..=1.0).contains(&offset) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Flanger offset {offset} is out of range"),
            ));
        }
        let offset = Ratio::new::<percent>(offset);

        let motion = reader.read_f32()?;
        let feedback = reader.read_f32()?;
        let spread = reader.read_f32()?;
        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let scroll = reader.read_bool32()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "flanger_unknown1")?;
        reader.expect_u32(0, "flanger_unknown2")?;

        if effect_version > 1002 {
            reader.expect_u32(0, "flanger_unknown3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Flanger {
                delay,
                depth,
                rate,
                scroll,
                offset,
                motion,
                spread,
                feedback,
                mix,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Flanger {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.delay.get::<second>())?;
        writer.write_f32(self.depth.get::<second>())?;
        writer.write_f32(self.rate)?;
        writer.write_f32(self.offset.get::<percent>())?;
        writer.write_f32(self.motion)?;
        writer.write_f32(self.feedback)?;
        writer.write_f32(self.spread)?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_bool32(self.scroll)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // flanger_unknown1
        writer.write_u32(0)?; // flanger_unknown2

        if self.write_version() > 1002 {
            writer.write_u32(0)?; // flanger_unknown3
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1013
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::time::second;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Flanger::default();
        assert_eq!(effect.delay.get::<second>(), 0.001);
        assert_relative_eq!(effect.depth.get::<second>(), 0.00103, epsilon = 0.00001);
        assert_eq!(effect.rate, 0.31);
        assert!(effect.scroll);
        assert_eq!(effect.offset_degrees(), 0.0);
        assert_relative_eq!(effect.motion, 0.5);
        assert_eq!(effect.spread, 0.25);
        assert_relative_eq!(effect.feedback, 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
    }

    #[test]
    fn disabled() {
        let preset = read_effect_preset("flanger", "flanger-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Flanger::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Flanger::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in [
            "flanger-1.8.0.phaseplant",
            "flanger-1.8.13.phaseplant",
            "flanger-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("flanger", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_flanger().unwrap();
            assert_eq!(effect.delay.get::<second>(), 0.001);

            assert_relative_eq!(effect.depth.get::<second>(), 0.00103, epsilon = 0.00001);
            assert_eq!(effect.rate, 0.31);
            assert!(effect.scroll);
            assert_eq!(effect.offset_degrees(), 0.0);
            assert_relative_eq!(effect.motion, 0.5);
            assert_eq!(effect.spread, 0.25);
            assert_relative_eq!(effect.feedback, 0.0);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "flanger",
            "flanger-feedback25-mix75-minimized-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_flanger().unwrap();
        assert_relative_eq!(effect.feedback, 0.25);
        assert_eq!(effect.mix.get::<percent>(), 75.0);

        let preset = read_effect_preset(
            "flanger",
            "flanger-offset45-motion2-spread50-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_flanger().unwrap();
        assert_eq!(effect.offset_degrees(), 45.0);
        assert_relative_eq!(effect.motion, 2.0, epsilon = 0.000001);
        assert_eq!(effect.spread, 0.5);

        let preset = read_effect_preset(
            "flanger",
            "flanger-scrolloff-delay7-depth5-rate2-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_flanger().unwrap();
        assert!(!effect.scroll);
        assert_relative_eq!(effect.delay.get::<second>(), 0.007, epsilon = 0.000001);
        assert_relative_eq!(effect.depth.get::<second>(), 0.005, epsilon = 0.000001);
        assert_eq!(effect.rate, 2.0);
    }
}

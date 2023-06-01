//! [Phaser](https://kilohearts.com/products/phaser) creates a sweeping sound.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1037           |
//! | 1.8.14              | 1037           |
//! | 2.0.16              | 1048           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};
use uom::si::f32::Ratio;
use uom::si::ratio::{percent, ratio};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Phaser {
    pub cutoff: f32,
    pub rate: f32,
    pub depth: f32,
    pub order: u32,
    pub spread: f32,
    pub mix: Ratio,
}

impl Default for Phaser {
    fn default() -> Self {
        Phaser {
            cutoff: 500.0,
            rate: 0.6,
            depth: 0.5,
            order: 3,
            spread: 0.5,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_phaser(&self) -> Option<&Phaser> {
        self.downcast_ref::<Phaser>()
    }
}

impl Effect for Phaser {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Phaser
    }
}

impl EffectRead for Phaser {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1037 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let enabled = reader.read_bool32()?;
        let order = reader.read_u32()?;
        let cutoff = reader.read_f32()?;
        let depth = reader.read_f32()?;
        let rate = reader.read_f32()?;
        let spread = reader.read_f32()?;
        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "phaser_end1")?;
        reader.expect_u32(0, "phaser_end2")?;
        if effect_version >= 1048 {
            reader.expect_u32(0, "phaser_end3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Phaser {
                cutoff,
                rate,
                depth,
                order,
                spread,
                mix,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Phaser {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_u32(self.order)?;
        writer.write_f32(self.cutoff)?;
        writer.write_f32(self.depth)?;
        writer.write_f32(self.rate)?;
        writer.write_f32(self.spread)?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        if self.write_version() >= 1048 {
            writer.write_u32(0)?;
        }
        Ok(())
    }

    fn write_version(&self) -> u32 {
        1037
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Phaser::default();
        assert_eq!(effect.cutoff, 500.0);
        assert_eq!(effect.rate, 0.6);
        assert_eq!(effect.depth, 0.5);
        assert_eq!(effect.order, 3);
        assert_eq!(effect.spread, 0.5);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
    }

    #[test]
    fn eq() {
        let effect = Phaser::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Phaser::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["phaser-1.8.13.phaseplant", "phaser-2.0.16.phaseplant"] {
            let preset = read_effect_preset("phaser", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_phaser().unwrap();
            assert_eq!(effect.cutoff, 500.0);
            assert_eq!(effect.rate, 0.6);
            assert_eq!(effect.depth, 0.5);
            assert_eq!(effect.order, 3);
            assert_eq!(effect.spread, 0.5);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "phaser",
            "phaser-cutoff250-rate1.2-depth25-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_phaser().unwrap();
        assert_relative_eq!(effect.cutoff, 250.0, epsilon = 0.0001);
        assert_eq!(effect.rate, 1.2);
        assert_eq!(effect.depth, 0.25);

        let preset =
            read_effect_preset("phaser", "phaser-order2-spread25-mix75-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_phaser().unwrap();
        assert_eq!(effect.order, 2);
        assert_eq!(effect.spread, 0.25);
        assert_eq!(effect.mix.get::<percent>(), 75.0);

        let preset =
            read_effect_preset("phaser", "phaser-order7-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_phaser().unwrap();
        assert_eq!(effect.order, 7);

        let preset =
            read_effect_preset("phaser", "phaser-rate6-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_phaser().unwrap();
        assert_eq!(effect.rate, 6.0);
    }
}

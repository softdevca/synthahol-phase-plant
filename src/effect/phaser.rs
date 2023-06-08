//! [Phaser](https://kilohearts.com/products/phaser) creates a sweeping sound.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.14     | 1037           |
//! | 2.0.0              | 1046           |
//! | 2.0.16              | 1048           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::percent;

use crate::SnapinId;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Phaser {
    pub cutoff: Frequency,
    pub rate: Frequency,
    pub depth: Ratio,
    pub order: u32,
    pub spread: Ratio,
    pub mix: Ratio,
}

impl Default for Phaser {
    fn default() -> Self {
        Self {
            cutoff: Frequency::new::<hertz>(500.0),
            rate: Frequency::new::<hertz>(0.6),
            depth: Ratio::new::<percent>(50.0),
            order: 3,
            spread: Ratio::new::<percent>(50.0),
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
        let cutoff = reader.read_hertz()?;
        let depth = reader.read_ratio()?;
        let rate = reader.read_hertz()?;
        let spread = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "phaser_unknown_1")?;
        reader.expect_u32(0, "phaser_unknown_2")?;

        let group_id = if effect_version >= 1046 {
            reader.read_snapin_position()?
        } else {
            None
        };

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
            group_id,
        ))
    }
}

impl EffectWrite for Phaser {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_u32(self.order)?;
        writer.write_hertz(self.cutoff)?;
        writer.write_ratio(self.depth)?;
        writer.write_hertz(self.rate)?;
        writer.write_ratio(self.spread)?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if self.write_version() >= 1048 {
            writer.write_snapin_id(group_id)?;
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
        assert_eq!(effect.cutoff.get::<hertz>(), 500.0);
        assert_eq!(effect.rate.get::<hertz>(), 0.6);
        assert_eq!(effect.depth.get::<percent>(), 50.0);
        assert_eq!(effect.order, 3);
        assert_eq!(effect.spread.get::<percent>(), 50.0);
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
            assert_eq!(effect.cutoff.get::<hertz>(), 500.0);
            assert_eq!(effect.rate.get::<hertz>(), 0.6);
            assert_eq!(effect.depth.get::<percent>(), 50.0);
            assert_eq!(effect.order, 3);
            assert_eq!(effect.spread.get::<percent>(), 50.0);
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
        assert_relative_eq!(effect.cutoff.get::<hertz>(), 250.0, epsilon = 0.0001);
        assert_eq!(effect.rate.get::<hertz>(), 1.2);
        assert_eq!(effect.depth.get::<percent>(), 25.0);

        let preset =
            read_effect_preset("phaser", "phaser-order2-spread25-mix75-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_phaser().unwrap();
        assert_eq!(effect.order, 2);
        assert_eq!(effect.spread.get::<percent>(), 25.0);
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
        assert_eq!(effect.rate.get::<hertz>(), 6.0);
    }
}

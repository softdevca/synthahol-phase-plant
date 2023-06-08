//! [Reverb](https://kilohearts.com/products/reverb) is a spatial simulation
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.17     | 1032           |
//! | 2.0.16              | 1049           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::{Ratio, Time};
use uom::si::ratio::percent;
use uom::si::time::second;

use crate::{Decibels, SnapinId};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Reverb {
    pub decay: Time,

    /// Decibels per second.
    pub dampen: Decibels,

    pub size: Ratio,
    pub width: Ratio,
    pub early: Ratio,
    pub mix: Ratio,
}

impl Default for Reverb {
    fn default() -> Self {
        Self {
            decay: Time::new::<second>(3.0),
            dampen: Decibels::new(25.0),
            size: Ratio::new::<percent>(100.0),
            width: Ratio::new::<percent>(100.0),
            early: Ratio::new::<percent>(25.0),
            mix: Ratio::new::<percent>(25.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_reverb(&self) -> Option<&Reverb> {
        self.downcast_ref::<Reverb>()
    }
}

impl Effect for Reverb {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Reverb
    }
}

impl EffectRead for Reverb {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1032 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let size = reader.read_ratio()?;
        let decay = reader.read_seconds()?;
        let dampen = reader.read_decibels_db()?;
        let width = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let early = reader.read_ratio()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "reverb_unknown_1")?;
        reader.expect_u32(0, "reverb_unknown_2")?;

        let group_id = if effect_version > 1032 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(Reverb {
                decay,
                dampen,
                size,
                width,
                early,
                mix,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Reverb {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_ratio(self.size)?;
        writer.write_seconds(self.decay)?;
        writer.write_decibels_db(self.dampen)?;
        writer.write_ratio(self.width)?;
        writer.write_ratio(self.mix)?;
        writer.write_ratio(self.early)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // reverb_unknown1
        writer.write_u32(0)?; // reverb_unknown2

        if self.write_version() > 1032 {
            writer.write_snapin_id(group_id)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1032
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
        let effect = Reverb::default();
        assert_eq!(effect.decay.get::<second>(), 3.0);
        assert_relative_eq!(effect.dampen.db(), 25.0, epsilon = 0.1);
        assert_eq!(effect.size.get::<percent>(), 100.0);
        assert_eq!(effect.width.get::<percent>(), 100.0);
        assert_relative_eq!(effect.early.get::<percent>(), 25.0, epsilon = 0.01);
        assert_relative_eq!(effect.mix.get::<percent>(), 25.0, epsilon = 0.01);
    }

    #[test]
    fn eq() {
        let effect = Reverb::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Reverb::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "reverb-1.7.0.phaseplant",
            "reverb-1.8.0.phaseplant",
            "reverb-1.8.13.phaseplant",
            "reverb-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("reverb", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_reverb().unwrap();
            assert_eq!(effect.decay.get::<second>(), 3.0);
            assert_relative_eq!(effect.dampen.db(), 25.0, epsilon = 0.1);
            assert_eq!(effect.size.get::<percent>(), 100.0);
            assert_eq!(effect.width.get::<percent>(), 100.0);
            assert_relative_eq!(effect.early.get::<percent>(), 24.8, epsilon = 0.1);
            assert_relative_eq!(effect.mix.get::<percent>(), 25.0, epsilon = 0.1);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset =
            read_effect_preset("reverb", "reverb-decay1-dampen30-size75%-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_reverb().unwrap();
        assert_eq!(effect.decay.get::<second>(), 1.0);
        assert_relative_eq!(effect.dampen.db(), 30.0, epsilon = 0.00001);
        assert_eq!(effect.size.get::<percent>(), 75.0);

        let preset = read_effect_preset(
            "reverb",
            "reverb-width50%-early60%-mix70%-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_reverb().unwrap();
        assert_relative_eq!(effect.width.get::<percent>(), 50.0);
        assert_relative_eq!(effect.early.get::<percent>(), 60.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 70.0, epsilon = 0.01);

        let preset =
            read_effect_preset("reverb", "reverb-decay30-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_reverb().unwrap();
        assert_relative_eq!(effect.decay.get::<second>(), 30.0, epsilon = 0.00001);

        let preset =
            read_effect_preset("reverb", "reverb-size50-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_reverb().unwrap();
        assert_relative_eq!(effect.size.get::<percent>(), 50.0);
    }
}

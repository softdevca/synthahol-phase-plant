//! [Limiter](https://kilohearts.com/products/limiter) is a volume threshold
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.16     | 1038           |
//! | 2.0.0               | 1047           |
//! | 2.0.12              | 1048           |

use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Time;
use uom::si::time::second;

use crate::effect::EffectVersion;
use crate::{Decibels, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Limiter {
    pub threshold: Decibels,
    pub release: Time,
    pub in_gain: Decibels,
    pub out_gain: Decibels,
}

impl Limiter {
    pub fn default_version() -> EffectVersion {
        1048
    }
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            threshold: Decibels::ZERO,
            release: Time::new::<second>(0.023),
            in_gain: Decibels::ZERO,
            out_gain: Decibels::ZERO,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_limiter(&self) -> Option<&Limiter> {
        self.downcast_ref::<Limiter>()
    }
}

impl Effect for Limiter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Limiter
    }
}

impl EffectRead for Limiter {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1038 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Limiter effect version {effect_version}"),
            ));
        }

        let enabled = reader.read_bool32()?;
        let in_gain = reader.read_decibels_linear()?;
        let out_gain = reader.read_decibels_linear()?;
        let threshold = reader.read_decibels_linear()?;
        let release = reader.read_seconds()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "limiter_unknown_1")?;
        reader.expect_u32(0, "limiter_unknown_2")?;

        let group_id = if effect_version >= 1047 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(Limiter {
                threshold,
                release,
                in_gain,
                out_gain,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Limiter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_decibels_linear(self.in_gain)?;
        writer.write_decibels_linear(self.out_gain)?;
        writer.write_decibels_linear(self.threshold)?;
        writer.write_seconds(self.release)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if snapin.effect_version >= 1047 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;
    use crate::Decibels;

    use super::*;

    #[test]
    fn default() {
        let effect = Limiter::default();
        assert_eq!(effect.in_gain, Decibels::ZERO);
        assert_eq!(effect.out_gain, Decibels::ZERO);
        assert_eq!(effect.threshold, Decibels::ZERO);
        assert_eq!(effect.release.get::<second>(), 0.023);
    }

    #[test]
    fn eq() {
        let effect = Limiter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Limiter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["limiter-1.8.13.phaseplant", "limiter-2.0.12.phaseplant"] {
            let preset = read_effect_preset("limiter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            assert_eq!(snapin.id, 1);
            let effect = snapin.effect.as_limiter().unwrap();
            assert_eq!(effect.in_gain, Decibels::ZERO);
            assert_eq!(effect.out_gain, Decibels::ZERO);
            assert_eq!(effect.threshold, Decibels::ZERO);
            assert_eq!(effect.release.get::<second>(), 0.023);
        }
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset("limiter", "limiter-in-5-out4-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_limiter().unwrap();
        assert_relative_eq!(effect.in_gain.db(), -5.0, epsilon = 0.0001);
        assert_relative_eq!(effect.out_gain.db(), 4.0, epsilon = 0.0001);

        let preset =
            read_effect_preset("limiter", "limiter-threshold3-release10-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_limiter().unwrap();
        assert_eq!(effect.release.get::<second>(), 0.010);
        assert_relative_eq!(effect.threshold.db(), 3.0, epsilon = 0.0001);

        let preset =
            read_effect_preset("limiter", "limiter-out10-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_limiter().unwrap();
        assert_eq!(effect.out_gain.db(), 10.0);

        let preset =
            read_effect_preset("limiter", "limiter-in10-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_limiter().unwrap();
        assert_eq!(effect.in_gain.db(), 10.0);
    }
}

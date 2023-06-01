//! [Limiter](https://kilohearts.com/products/limiter) is a volume threshold
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.16     | 1038           |
//! | 2.0.12              | 1048           |

use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::Decibels;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Limiter {
    pub threshold: f32,
    pub release: f32,
    pub in_gain: f32,
    pub out_gain: f32,
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            threshold: Decibels::ZERO.linear(),
            release: 0.023,
            in_gain: Decibels::ZERO.linear(),
            out_gain: Decibels::ZERO.linear(),
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
        let in_gain = reader.read_f32()?;
        let out_gain = reader.read_f32()?;
        let threshold = reader.read_f32()?;
        let release = reader.read_f32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "limiter_unknown_1")?;
        reader.expect_u32(0, "limiter_unknown_2")?;
        if effect_version >= 1048 {
            reader.expect_u32(0, "limiter_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Limiter {
                threshold,
                release,
                in_gain,
                out_gain,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Limiter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.in_gain)?;
        writer.write_f32(self.out_gain)?;
        writer.write_f32(self.threshold)?;
        writer.write_f32(self.release)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        if self.write_version() >= 1048 {
            writer.write_u32(0)?;
        }
        Ok(())
    }

    fn write_version(&self) -> u32 {
        1048
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
        assert_eq!(effect.in_gain, Decibels::ZERO.linear());
        assert_eq!(effect.out_gain, Decibels::ZERO.linear());
        assert_eq!(effect.threshold, Decibels::ZERO.linear());
        assert_eq!(effect.release, 0.023);
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
            assert_eq!(snapin.position, 1);
            let effect = snapin.effect.as_limiter().unwrap();
            assert_eq!(effect.in_gain, Decibels::ZERO.linear());
            assert_eq!(effect.out_gain, Decibels::ZERO.linear());
            assert_eq!(effect.threshold, Decibels::ZERO.linear());
            assert_eq!(effect.release, 0.023);
        }
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset("limiter", "limiter-in-5-out4-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_limiter().unwrap();
        assert_relative_eq!(
            effect.in_gain,
            Decibels::new(-5.0).linear(),
            epsilon = 0.0001
        );
        assert_relative_eq!(
            effect.out_gain,
            Decibels::new(4.0).linear(),
            epsilon = 0.0001
        );

        let preset =
            read_effect_preset("limiter", "limiter-threshold3-release10-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_limiter().unwrap();
        assert_eq!(effect.release, 0.010);
        assert_relative_eq!(
            effect.threshold,
            Decibels::new(3.0).linear(),
            epsilon = 0.0001
        );

        let preset =
            read_effect_preset("limiter", "limiter-out10-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_limiter().unwrap();
        assert_eq!(effect.out_gain, Decibels::new(10.0).linear());

        let preset =
            read_effect_preset("limiter", "limiter-in10-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_limiter().unwrap();
        assert_eq!(effect.in_gain, Decibels::new(10.0).linear());
    }
}

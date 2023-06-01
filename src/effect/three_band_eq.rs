//! [3-Band Eq](https://kilohearts.com/products/3band_eq) is a simple equalizer
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1015           |
//! | 1.8.15              | 1015           |
//! | 2.0.12              | 1025           |
//! | 2.0.16              | 1026           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Frequency;
use uom::si::frequency::hertz;

use crate::Decibels;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct ThreeBandEq {
    #[doc(alias = "low_split")]
    pub low_freq: Frequency,

    #[doc(alias = "high_split")]
    pub high_freq: Frequency,

    pub low_gain: Decibels,
    pub mid_gain: Decibels,
    pub high_gain: Decibels,
}

impl Default for ThreeBandEq {
    fn default() -> Self {
        ThreeBandEq {
            low_freq: Frequency::new::<hertz>(220.0),
            high_freq: Frequency::new::<hertz>(2200.0),
            low_gain: Decibels::ZERO,
            mid_gain: Decibels::ZERO,
            high_gain: Decibels::ZERO,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_three_band_eq(&self) -> Option<&ThreeBandEq> {
        self.downcast_ref::<ThreeBandEq>()
    }
}

impl Effect for ThreeBandEq {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::ThreeBandEq
    }
}

impl EffectRead for ThreeBandEq {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1015 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let low_gain = Decibels::new(reader.read_f32()?);
        let mid_gain = Decibels::new(reader.read_f32()?);
        let high_gain = Decibels::new(reader.read_f32()?);
        let low_freq = Frequency::new::<hertz>(reader.read_f32()?);
        let high_freq = Frequency::new::<hertz>(reader.read_f32()?);
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "three_band_eq_unknown1")?;
        reader.expect_u32(0, "three_band_eq_unknown2")?;
        if effect_version >= 1025 {
            reader.expect_u32(0, "three_band_eq_unknown3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(ThreeBandEq {
                low_freq,
                high_freq,
                low_gain,
                mid_gain,
                high_gain,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for ThreeBandEq {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.low_gain.db())?;
        writer.write_f32(self.mid_gain.db())?;
        writer.write_f32(self.high_gain.db())?;
        writer.write_f32(self.low_freq.get::<hertz>())?;
        writer.write_f32(self.high_freq.get::<hertz>())?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // three_band_eq_unknown1
        writer.write_u32(0)?; // three_band_eq_unknown2
        if self.write_version() >= 1025 {
            writer.write_u32(0)?; // three_band_eq_unknown3
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1026
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::f32::Frequency;
    use uom::si::frequency::hertz;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;
    use crate::Decibels;

    use super::*;

    #[test]
    fn default() {
        let effect = ThreeBandEq::default();
        assert_eq!(effect.low_gain, Decibels::ZERO);
        assert_eq!(effect.mid_gain, Decibels::ZERO);
        assert_eq!(effect.high_gain, Decibels::ZERO);
        assert_eq!(effect.low_freq, Frequency::new::<hertz>(220.0));
        assert_eq!(effect.high_freq, Frequency::new::<hertz>(2200.0));
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("three_band_eq", "three_band_eq-disabled-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = ThreeBandEq::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, ThreeBandEq::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "three_band_eq-1.8.13.phaseplant",
            "three_band_eq-2.0.12.phaseplant",
            "three_band_eq-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("three_band_eq", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_three_band_eq().unwrap();
            assert_eq!(&ThreeBandEq::default(), effect);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("three_band_eq", "three_band_eq-minimized-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "three_band_eq",
            "three_band_eq-100hz-3khz-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_three_band_eq().unwrap();
        assert_relative_eq!(effect.low_freq.get::<hertz>(), 100.0);
        assert_relative_eq!(effect.high_freq.get::<hertz>(), 3000.0);

        let preset =
            read_effect_preset("three_band_eq", "three_band_eq--10--1-9-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_three_band_eq().unwrap();
        assert_eq!(effect.low_gain.db(), -10.0);
        assert_eq!(effect.mid_gain.db(), -1.0);
        assert_eq!(effect.high_gain.db(), 9.0);

        let preset = read_effect_preset(
            "three_band_eq",
            "three_band_eq-bass_boost-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Bass Boost");
        assert_eq!(snapin.preset_path, vec!["factory", "Bass Boost.ksqe"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_three_band_eq().unwrap();
        assert_relative_eq!(effect.low_freq.get::<hertz>(), 220.0);
    }
}

//! [Chorus](https://kilohearts.com/products/chorus) creates a stereo effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.6.9 10. 1.8.16    | 1037           |
//! | 2.0.12              | 1047           |
//! | 2.0.16              | 1048           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};
use uom::si::f32::{Frequency, Ratio, Time};
use uom::si::frequency::hertz;
use uom::si::ratio::{percent, ratio};
use uom::si::time::{millisecond, second};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Chorus {
    pub taps: u8,
    pub mix: Ratio,
    pub spread: Ratio,
    pub delay: Time,
    pub depth: Time,
    pub rate: Frequency,
}

impl Chorus {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Chorus {
    fn default() -> Self {
        Chorus {
            taps: 2,
            mix: Ratio::new::<percent>(100.0),
            spread: Ratio::new::<percent>(100.0),
            delay: Time::new::<millisecond>(4.0),
            depth: Time::new::<millisecond>(4.0),
            rate: Frequency::new::<hertz>(0.6),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_chorus(&self) -> Option<&Chorus> {
        self.downcast_ref::<Chorus>()
    }
}

impl Effect for Chorus {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Chorus
    }
}

impl EffectRead for Chorus {
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
        let delay = Time::new::<second>(reader.read_f32()?);
        let rate = Frequency::new::<hertz>(reader.read_f32()?);
        let depth = Time::new::<second>(reader.read_f32()?);
        let spread = Ratio::new::<ratio>(reader.read_f32()?);
        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let taps = match reader.read_u32()? {
            0 => Ok(2),
            1 => Ok(3),
            taps => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Unexpected number of taps {taps} ({taps:#x}) at position {}",
                    reader.stream_position()? - 4
                ),
            )),
        }?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "chorus_unknown_1")?;
        reader.expect_u32(0, "chorus_unknown_2")?;
        if effect_version >= 1047 {
            reader.expect_u32(0, "chorus_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Chorus {
                taps,
                mix,
                spread,
                delay,
                depth,
                rate,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Chorus {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.delay.get::<second>())?;
        writer.write_f32(self.rate.get::<hertz>())?;
        writer.write_f32(self.depth.get::<second>())?;
        writer.write_f32(self.spread.get::<ratio>())?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_u32(self.taps as u32 - 2)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // chorus_unknown_1
        writer.write_u32(0)?; // chorus_unknown_2
        writer.write_u32(0)?; // chorus_unknown_3

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

    use super::*;

    #[test]
    fn default() {
        let effect = Chorus::default();
        assert_eq!(effect.taps, 2);
        assert_eq!(effect.delay.get::<millisecond>(), 4.0);
        assert_eq!(effect.depth.get::<millisecond>(), 4.0);
        assert_eq!(effect.rate.get::<hertz>(), 0.6);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.spread.get::<percent>(), 100.0);
    }

    #[test]
    fn eq() {
        let effect = Chorus::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Chorus::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in [
            "chorus-1.8.13.phaseplant",
            "chorus-2.0.12.phaseplant",
            "chorus-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("chorus", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            assert!(snapin.preset_path.is_empty());
            let effect = snapin.effect.as_chorus().unwrap();
            assert_eq!(effect.taps, 2); // Used for the default
            assert_eq!(effect.delay.get::<millisecond>(), 4.0);
            assert_eq!(effect.depth.get::<millisecond>(), 4.0);
            assert_eq!(effect.rate.get::<hertz>(), 0.6);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
            assert_eq!(effect.spread.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset("chorus", "chorus-disabled-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);

        let preset = read_effect_preset("chorus", "chorus-madness-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Madness");
        assert_eq!(snapin.preset_path, vec!["factory", "Madness.ksch"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_chorus().unwrap();
        assert_relative_eq!(effect.delay.get::<millisecond>(), 1.23, epsilon = 0.01);

        let preset = read_effect_preset("chorus", "chorus-minimized-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);

        let preset = read_effect_preset("chorus", "chorus-spread-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_chorus().unwrap();
        assert_eq!(effect.spread.get::<percent>(), 50.0);
    }

    #[test]
    fn parts_version_2() {
        let preset =
            read_effect_preset("chorus", "chorus-taps-rate-disabled-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_chorus().unwrap();
        assert_eq!(effect.taps, 3);
        assert_relative_eq!(effect.rate.get::<hertz>(), 2.0);
    }
}

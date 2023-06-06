//! [Reverser](https://kilohearts.com/products/reverser) is a reversed echo.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.13     | 1033           |
//! | 2.0.16              | 1044           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::{Ratio, Time};
use uom::si::ratio::percent;
use uom::si::time::{millisecond, second};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Reverser {
    pub time: Time,
    pub sync: bool,
    pub crossfade: Ratio,
    pub mix: Ratio,
    unknown2: u32,
    unknown3: u32,
}

impl Default for Reverser {
    fn default() -> Self {
        Self {
            time: Time::new::<millisecond>(200.0),
            sync: true,
            crossfade: Ratio::new::<percent>(10.0),
            mix: Ratio::new::<percent>(50.0),
            unknown2: 4,
            unknown3: 4,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_reverser(&self) -> Option<&Reverser> {
        self.downcast_ref::<Reverser>()
    }
}

impl Effect for Reverser {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Reverser
    }
}

impl EffectRead for Reverser {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1033 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let time = reader.read_seconds()?;

        let unknown2 = reader.read_u32()?;
        let unknown3 = reader.read_u32()?;

        let sync = reader.read_bool32()?;
        let mix = reader.read_ratio()?;
        let crossfade = reader.read_ratio()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "reverser_unknown_1")?;
        reader.expect_u32(0, "reverser_unknown_2")?;
        if effect_version > 1038 {
            reader.expect_u32(0, "reverser_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Reverser {
                time,
                sync,
                crossfade,
                mix,
                unknown2,
                unknown3,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Reverser {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.time.get::<second>())?;

        writer.write_u32(self.unknown2)?;
        writer.write_u32(self.unknown3)?;

        writer.write_bool32(self.sync)?;
        writer.write_ratio(self.mix)?;
        writer.write_ratio(self.crossfade)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        if self.write_version() > 1038 {
            writer.write_u32(0)?;
        }
        Ok(())
    }

    fn write_version(&self) -> u32 {
        1044
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
        let effect = Reverser::default();
        assert_eq!(effect.time.get::<second>(), 0.2);
        assert!(effect.sync);
        assert_relative_eq!(effect.crossfade.get::<percent>(), 10.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 50.0);
    }

    #[test]
    fn eq() {
        let effect = Reverser::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Reverser::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["reverser-1.8.13.phaseplant", "reverser-2.0.16.phaseplant"] {
            let preset = read_effect_preset("reverser", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_reverser().unwrap();
            assert!(effect.sync);
            assert_relative_eq!(effect.time.get::<millisecond>(), 200.0, epsilon = 0.001);
            assert_relative_eq!(effect.crossfade.get::<percent>(), 10.0, epsilon = 0.001);
            assert_relative_eq!(effect.mix.get::<percent>(), 50.0, epsilon = 0.001);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "reverser",
            "reverser-100ms-crossfade25-mix33-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_reverser().unwrap();
        assert_relative_eq!(effect.time.get::<millisecond>(), 100.0, epsilon = 0.001);
        assert_relative_eq!(effect.crossfade.get::<percent>(), 25.0, epsilon = 0.001);
        assert_relative_eq!(effect.mix.get::<percent>(), 33.0, epsilon = 0.001);
    }
}

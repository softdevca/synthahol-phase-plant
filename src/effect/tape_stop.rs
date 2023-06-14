//! [Tape Stop](https://kilohearts.com/products/tape_stop) a tape speed
//! simulation effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.14     | 1034           |
//! | 2.0.16              | 1045           |

use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::effect::EffectVersion;
use uom::si::f32::Time;
use uom::si::time::second;

use crate::Snapin;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct TapeStop {
    pub running: bool,
    pub stop_time: Time,
    pub start_time: Time,
    pub curve: f32,
}

impl TapeStop {
    pub fn default_version() -> EffectVersion {
        1045
    }
}

impl Default for TapeStop {
    fn default() -> Self {
        Self {
            running: true,
            stop_time: Time::new::<second>(0.2),
            start_time: Time::new::<second>(0.2),
            curve: 1.0,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_tape_stop(&self) -> Option<&TapeStop> {
        self.downcast_ref::<TapeStop>()
    }
}

impl Effect for TapeStop {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::TapeStop
    }
}

impl EffectRead for TapeStop {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1034 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("TapeStop effect version {effect_version}"),
            ));
        }

        let running = reader.read_bool32()?;
        let start_time = reader.read_seconds()?;
        let stop_time = reader.read_seconds()?;
        let enabled = reader.read_bool32()?;
        let curve = reader.read_f32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "tape_stop_unknown2")?;
        reader.expect_u32(0, "tape_stop_unknown3")?;

        let group_id = if effect_version > 1038 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(TapeStop {
                running,
                stop_time,
                start_time,
                curve,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for TapeStop {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(self.running)?;
        writer.write_seconds(self.start_time)?;
        writer.write_seconds(self.stop_time)?;
        writer.write_bool32(snapin.enabled)?;
        writer.write_f32(self.curve)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if snapin.effect_version > 1038 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        Ok(())
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
        let effect = TapeStop::default();
        assert!(effect.running);
        assert_eq!(effect.start_time.get::<second>(), 0.2);
        assert_eq!(effect.stop_time.get::<second>(), 0.2);
        assert_eq!(effect.curve, 1.0);
    }

    #[test]
    fn eq() {
        let effect = TapeStop::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, TapeStop::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["tape_stop-1.8.14.phaseplant", "tape_stop-2.0.16.phaseplant"] {
            let preset = read_effect_preset("tape_stop", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_tape_stop().unwrap();
            assert!(effect.running);
            assert_eq!(effect.start_time.get::<second>(), 0.2);
            assert_eq!(effect.stop_time.get::<second>(), 0.2);
            assert_eq!(effect.curve, 1.0);
        }
    }

    #[test]
    fn curve_disabled() {
        let preset =
            read_effect_preset("tape_stop", "tape_stop-curve3-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_tape_stop().unwrap();
        assert_eq!(effect.curve, 3.0);
    }

    #[test]
    fn start_minimized() {
        let preset =
            read_effect_preset("tape_stop", "tape_stop-start2-minimized-1.8.14.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_tape_stop().unwrap();
        assert_relative_eq!(effect.start_time.get::<second>(), 2.0, epsilon = 0.0001);
    }

    #[test]
    fn times_stopped() {
        let preset = read_effect_preset(
            "tape_stop",
            "tape_stop-start150-stop350-stopped-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_tape_stop().unwrap();
        assert_eq!(effect.start_time.get::<second>(), 0.350);
        assert_eq!(effect.stop_time.get::<second>(), 0.150);
        assert!(!effect.running);
    }
}

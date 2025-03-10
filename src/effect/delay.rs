//! [Delay](https://kilohearts.com/products/delay) is an echo effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.14     | 1037           |
//! | 2.0.0               | 1046           |
//! | 2.0.12              | 1049           |
//! | 2.1.16 to 2.1.0     | 1050           |

// The tone control was added in Phase Plant 2.0.9.

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::effect::EffectVersion;
use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::{percent, ratio};
use uom::si::time::second;

use crate::Snapin;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug)]
pub struct Delay {
    pub time: Time,

    /// If the the delay isn't using sync mode then the delay is based on time
    /// instead of rhythm.
    pub sync: bool,

    pub feedback: Ratio,

    /// Bounce was called Ping Pong prior to Phase Plant version 2.
    #[doc(alias = "ping pong")]
    pub bounce: bool,

    pub duck: Ratio,
    pub pan: Ratio,
    pub mix: Ratio,
    pub tone: Ratio,
    unknown2: u32,
    unknown3: u32,
}

impl Delay {
    pub fn default_version() -> EffectVersion {
        1050
    }
}

impl PartialEq for Delay {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
            && self.sync == other.sync
            && self.feedback == other.feedback
            && self.bounce == other.bounce
            && self.duck == other.duck
            && self.pan == other.pan
            && self.mix == other.mix
            && self.tone == other.tone
    }
}

impl Default for Delay {
    /// The default feedback in Phase Plant version 2.0 is 0.4972512 but in
    /// version 1 it is 0.5. The latter is used because the version 2 value
    /// may not have been intended by Kilohearts.
    fn default() -> Self {
        Self {
            time: Time::new::<second>(0.2),
            sync: false,
            feedback: Ratio::new::<percent>(50.0),
            bounce: false,
            duck: Ratio::zero(),
            pan: Ratio::zero(),
            mix: Ratio::new::<percent>(50.0),
            tone: Ratio::zero(),
            unknown2: 3,
            unknown3: 4,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_delay(&self) -> Option<&Delay> {
        self.downcast_ref::<Delay>()
    }
}

impl Effect for Delay {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Delay
    }
}

impl EffectRead for Delay {
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
        let time = reader.read_seconds()?;

        let unknown2 = reader.read_u32()?;
        let unknown3 = reader.read_u32()?;

        let sync = reader.read_bool32()?;
        let feedback = reader.read_ratio()?;
        let pan = reader.read_ratio()?;
        let bounce = reader.read_bool32()?;
        let duck = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "delay_unknown_5")?;
        reader.expect_u32(0, "delay_unknown_6")?;

        let group_id = if effect_version >= 1046 {
            reader.read_snapin_position()?
        } else {
            None
        };

        let mut tone = Ratio::zero();
        if effect_version >= 1049 {
            tone = reader.read_ratio()?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Delay {
                time,
                sync,
                feedback,
                bounce,
                duck,
                pan,
                mix,
                tone,
                unknown2,
                unknown3,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Delay {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_f32(self.time.get::<second>())?;

        writer.write_u32(self.unknown2)?;
        writer.write_u32(self.unknown3)?;

        writer.write_bool32(self.sync)?;
        writer.write_f32(self.feedback.get::<ratio>())?;
        writer.write_f32(self.pan.get::<ratio>())?;
        writer.write_bool32(self.bounce)?;
        writer.write_f32(self.duck.get::<ratio>())?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if snapin.effect_version >= 1049 {
            writer.write_snapin_id(snapin.group_id)?;
            writer.write_f32(self.tone.get::<ratio>())?;
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
        let effect = Delay::default();
        assert_eq!(effect.time.get::<second>(), 0.200);
        assert_relative_eq!(effect.feedback.get::<percent>(), 50.0);
        assert!(!effect.sync);
        assert!(!effect.bounce);
        assert_relative_eq!(effect.duck.get::<percent>(), 0.0);
        assert_relative_eq!(effect.pan.get::<percent>(), 0.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 50.0);
    }

    #[test]
    fn eq() {
        let effect = Delay::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Delay::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init_version_1() {
        let preset = read_effect_preset("delay", "delay-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_delay().unwrap();
        assert_eq!(&Delay::default(), effect);
    }

    #[test]
    fn init_version_2() {
        for file in [
            "delay-2.0.12.phaseplant",
            "delay-2.0.16.phaseplant",
            "delay-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("delay", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_delay().unwrap();

            // Cannot compare against the default because Phase Plant version 2
            // has a slightly different default feedback.
            assert_eq!(effect.time.get::<second>(), 0.200);
            assert_relative_eq!(effect.feedback.get::<percent>(), 49.72512);
            assert!(!effect.sync);
            assert!(!effect.bounce);
            assert_relative_eq!(effect.duck.get::<percent>(), 0.0);
            assert_relative_eq!(effect.pan.get::<percent>(), 0.0);
            assert_relative_eq!(effect.mix.get::<percent>(), 50.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "delay",
            "delay-111ms-sync-feedback75%-mix45%-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_delay().unwrap();
        assert_relative_eq!(effect.time.get::<second>(), 0.111, epsilon = 0.00001);
        assert_eq!(effect.feedback.get::<percent>(), 75.0);
        assert!(effect.sync);
        assert_eq!(effect.mix.get::<percent>(), 45.0);

        let preset = read_effect_preset(
            "delay",
            "delay-ping_pong-duck11%-pan77%left-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_delay().unwrap();
        assert_relative_eq!(effect.time.get::<second>(), 0.200, epsilon = 0.00001);
        // 3/16
        assert_relative_eq!(effect.feedback.get::<percent>(), 50.0, epsilon = 0.5);
        assert!(effect.bounce);
        assert_eq!(effect.duck.get::<percent>(), 11.0);
        assert_eq!(effect.pan.get::<percent>(), -77.0);

        let preset =
            read_effect_preset("delay", "delay-ping_pong-disabled-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_delay().unwrap();
        assert!(effect.bounce);

        let preset =
            read_effect_preset("delay", "delay-ping_pong132-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        assert_eq!(snapin.preset_name, "Ping-Pong 1.32");
        assert_eq!(
            snapin.preset_path,
            vec!["factory", "Ping-Pong", "Ping-Pong 1.32.ksdl"]
        );
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_delay().unwrap();
        assert!(effect.bounce);
    }

    #[test]
    fn tone() {
        let preset = read_effect_preset("delay", "delay-tone25-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_delay().unwrap();
        assert_relative_eq!(effect.tone.get::<percent>(), 25.0);
    }
}

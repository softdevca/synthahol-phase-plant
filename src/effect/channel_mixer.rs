//! [Channel Mixer](https://kilohearts.com/products/channel_mixer) is a stereo
//! and phase utility.
//!
//! Channel Mixer was added to Phase Plant in version 2.0.8.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 2.0.16              | 1002           |

use crate::effect::EffectVersion;
use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};
use std::ops::RangeInclusive;

use crate::Snapin;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelMixer {
    pub left_to_left: f32,
    pub left_to_right: f32,
    pub right_to_left: f32,
    pub right_to_right: f32,
}

impl ChannelMixer {
    /// The minimum and maximum values for the mix levels.
    pub const MIX_RANGE: RangeInclusive<f32> = -1.0..=1.0;

    pub fn default_version() -> EffectVersion {
        1002
    }
}

impl Default for ChannelMixer {
    fn default() -> Self {
        Self {
            left_to_left: 1.0,
            left_to_right: 0.0,
            right_to_left: 0.0,
            right_to_right: 1.0,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_channel_mixer(&self) -> Option<&ChannelMixer> {
        self.downcast_ref::<ChannelMixer>()
    }
}

impl Effect for ChannelMixer {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::ChannelMixer
    }
}

impl EffectRead for ChannelMixer {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version > 1002 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Channel Mixer effect version {effect_version}"),
            ));
        }

        let left_to_left = reader.read_f32()?;
        if !ChannelMixer::MIX_RANGE.contains(&left_to_left) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Left to left value of {left_to_left} is out of range"),
            ));
        }

        let right_to_left = reader.read_f32()?;
        if !ChannelMixer::MIX_RANGE.contains(&right_to_left) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Right to to left value of {right_to_left} is out of range"),
            ));
        }

        let left_to_right = reader.read_f32()?;
        if !ChannelMixer::MIX_RANGE.contains(&left_to_right) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Left to right value of {left_to_right} is out of range"),
            ));
        }

        let right_to_right = reader.read_f32()?;
        if !ChannelMixer::MIX_RANGE.contains(&right_to_right) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Right to right value of {right_to_right} is out of range"),
            ));
        }

        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "channel_mixer_unknown_1")?;
        reader.expect_u32(0, "channel_mixer_unknown_2")?;

        let group_id = reader.read_snapin_position()?;

        Ok(EffectReadReturn::new(
            Box::new(ChannelMixer {
                left_to_left,
                left_to_right,
                right_to_left,
                right_to_right,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for ChannelMixer {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_f32(self.left_to_left)?;
        writer.write_f32(self.right_to_left)?;
        writer.write_f32(self.left_to_right)?;
        writer.write_f32(self.right_to_right)?;

        writer.write_u32(0)?; // channel_mixer_unknown_1
        writer.write_u32(0)?; // channel_mixer_unknown_2

        writer.write_snapin_id(snapin.group_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn defaults() {
        let effect = ChannelMixer::default();
        assert_eq!(effect.left_to_left, 1.0);
        assert_eq!(effect.right_to_left, 0.0);
        assert_eq!(effect.left_to_right, 0.0);
        assert_eq!(effect.right_to_right, 1.0);
    }

    #[test]
    fn eq() {
        let effect = ChannelMixer::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, ChannelMixer::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        let preset =
            read_effect_preset("channel_mixer", "channel_mixer-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        assert_eq!(snapin.id, 1);
        let effect = snapin.effect.as_channel_mixer().unwrap();
        assert_eq!(effect, &Default::default());
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("channel_mixer", "channel_mixer-minimized-2.0.16.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset(
            "channel_mixer",
            "channel_mixer-ltol50-rtol-50-disabled-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_channel_mixer().unwrap();
        assert_relative_eq!(effect.left_to_left, 0.50, epsilon = 0.01);
        assert_relative_eq!(effect.left_to_right, 0.0, epsilon = 0.01);
        assert_relative_eq!(effect.right_to_left, -0.50, epsilon = 0.01);
        assert_relative_eq!(effect.right_to_right, 1.0, epsilon = 0.01);

        let preset = read_effect_preset(
            "channel_mixer",
            "channel_mixer-ltor50-rtor75-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_channel_mixer().unwrap();
        assert_relative_eq!(effect.left_to_left, 1.0, epsilon = 0.01);
        assert_relative_eq!(effect.left_to_right, 0.5, epsilon = 0.01);
        assert_relative_eq!(effect.right_to_left, 0.0, epsilon = 0.01);
        assert_relative_eq!(effect.right_to_right, 0.75, epsilon = 0.01);
    }
}

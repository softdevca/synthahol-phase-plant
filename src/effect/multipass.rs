//! [Multipass](https://kilohearts.com/products/multipass) is a band-splitting
//! host for effects.
//!
//! The ability to add Multipass as an effect was added in Phase Plant 1.8.0.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.0 to 1.8.5      | 1044           |
//! | 2.0.0               | 1056           |
//! | 2.0.12              | 1057           |
//! | 2.1.0               | 1058           |

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use log::trace;
use strum_macros::EnumIter;
use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::effect::EffectVersion;
use crate::{Decibels, MacroControl, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Debug, PartialEq)]
pub struct Lane {
    pub enabled: bool,

    /// There is no restriction on the number of snapins.
    pub snapins: Vec<Snapin>,

    pub mute: bool,
    pub solo: bool,
    pub gain: Decibels,
    pub mix: Ratio,
    pub pan: Ratio,
    pub post: Ratio,
}

impl Lane {
    pub const COUNT: usize = 7;
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            enabled: true,
            snapins: Vec::new(),
            mute: false,
            solo: false,
            gain: Decibels::from_linear(1.0),
            mix: Ratio::new::<percent>(100.0),
            pan: Ratio::new::<percent>(0.0),
            post: Ratio::new::<percent>(100.0),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, PartialEq)]
#[repr(u8)]
pub enum ExternalInputMode {
    Off,
    Sideband,
}

impl ExternalInputMode {
    // pub(crate) fn from_name(name: &str) -> Result<ExternalInputMode, Error> {
    //     match ExternalInputMode::iter().find(|mode| mode.to_string() == name) {
    //         Some(mode) => Ok(mode),
    //         None => Err(Error::new(
    //             ErrorKind::InvalidData,
    //             format!("External input mode '{name}' not found"),
    //         )),
    //     }
    // }
}

impl Display for ExternalInputMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ExternalInputMode::Off => "Off",
            ExternalInputMode::Sideband => "Sideband",
        };
        f.write_str(msg)
    }
}

#[derive(Debug, PartialEq)]
pub struct Multipass {
    pub name: Option<String>,
    pub gain: Decibels,
    pub pan: Ratio,
    pub mix: Ratio,
    pub external_input_mode: ExternalInputMode,
    pub lanes: [Lane; Lane::COUNT],
    pub macro_controls: [MacroControl; MacroControl::COUNT],
}

impl Multipass {
    pub fn default_version() -> EffectVersion {
        1058
    }
}

impl Default for Multipass {
    fn default() -> Self {
        Self {
            name: None,
            gain: Decibels::ZERO,
            pan: Ratio::new::<percent>(50.0),
            mix: Ratio::new::<percent>(100.0),
            external_input_mode: ExternalInputMode::Off,
            lanes: Default::default(),
            macro_controls: MacroControl::defaults(),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_multipass(&self) -> Option<&Multipass> {
        self.downcast_ref::<Multipass>()
    }
}

impl Effect for Multipass {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Multipass
    }
}

impl EffectRead for Multipass {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1044 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        // FIXME: Metadata is before the start.

        let mut effect = Multipass::default();
        let preset_name = reader.read_string_and_length()?;
        let preset_path = reader.read_path()?;
        let preset_edited = reader.read_bool8()?; // FIXME: Guess

        trace!(
            "multipass: preset name {preset_name:?}, path {preset_path:?}, edited {preset_edited}"
        );

        let enabled = true;
        let group_id = None;

        reader.expect_bool32(true, "multipass_1")?;
        reader.skip(20)?;

        trace!("multipass: lanes pos {}", reader.pos());
        for mut lane in &mut effect.lanes {
            reader.skip(4)?; // FIXME: Decode.
            lane.gain = reader.read_decibels_linear()?;
            lane.pan = reader.read_ratio()?;
            lane.mix = reader.read_ratio()?;
            lane.post = reader.read_ratio()?;
            reader.skip(4)?;
            reader.skip(4)?;
            trace!("multipass: lane {lane:?}");
        }

        for mut macro_control in &mut effect.macro_controls {
            macro_control.value = reader.read_f32()?;
        }

        reader.skip(1696 - 276)?;

        let minimized = reader.read_bool32()?;
        reader.skip(272)?;

        if effect_version >= 1056 {
            reader.skip(10239 - 149)?;
        }

        if effect_version >= 1057 {
            reader.skip(149)?;
        }

        if effect_version >= 1058 {
            // See Snap Heap for this 128 block, might be the same
            reader.skip(128)?;
        }

        // for macro_control in &mut effect.macro_controls {
        //     macro_control.name = reader.read_string_and_length()?.unwrap_or_default();
        // }

        Ok(EffectReadReturn {
            effect: Box::new(effect),
            enabled,
            minimized,
            group_id,
            metadata: Default::default(),
            preset_name,
            preset_path,
            preset_edited,
        })
    }
}

impl EffectWrite for Multipass {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_string_and_length(&snapin.preset_name)?;
        writer.write_path(&snapin.preset_path)?;
        writer.write_bool8(snapin.preset_edited)?;

        // TODO: Finish writing Multipass

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
    fn default() {
        let effect = Multipass::default();
        assert_eq!(effect.gain.db(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.external_input_mode, ExternalInputMode::Off);
        assert_eq!(effect.macro_controls[0].name, "Macro 1");
        assert_eq!(effect.macro_controls[0].value, 0.0);
        assert_eq!(effect.macro_controls[1].name, "Macro 2");
        assert_eq!(effect.macro_controls[1].value, 0.0);
        assert_eq!(effect.macro_controls[2].name, "Macro 3");
        assert_eq!(effect.macro_controls[2].value, 0.0);
        assert_eq!(effect.macro_controls[3].name, "Macro 4");
        assert_eq!(effect.macro_controls[3].value, 0.0);
        assert_eq!(effect.macro_controls[4].name, "Macro 5");
        assert_eq!(effect.macro_controls[4].value, 0.0);
        assert_eq!(effect.macro_controls[5].name, "Macro 6");
        assert_eq!(effect.macro_controls[5].value, 0.0);
        assert_eq!(effect.macro_controls[6].name, "Macro 7");
        assert_eq!(effect.macro_controls[6].value, 0.0);
        assert_eq!(effect.macro_controls[7].name, "Macro 8");
        assert_eq!(effect.macro_controls[7].value, 0.0);
    }

    #[test]
    fn eq() {
        let effect = Multipass::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Multipass::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    pub fn init() {
        for file in &[
            "multipass-1.8.0.phaseplant",
            "multipass-1.8.5.phaseplant",
            "multipass-2.0.12.phaseplant",
            "multipass-2.0.16.phaseplant",
            "multipass-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("multipass", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_multipass().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    /// Each lane has a gain of 10 dB, pan of 20%, mix of 30%, and a post of
    /// 40% if available.
    #[test]
    pub fn lanes_gain_pan_mix_post() {
        let preset = read_effect_preset(
            "multipass",
            "multipass-lanes-gain10-pan20-mix30-post40-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_multipass().unwrap();

        let pre_fx = &effect.lanes[5];
        assert_relative_eq!(pre_fx.gain.db(), 10.0, epsilon = 0.001);
        assert_relative_eq!(pre_fx.pan.get::<percent>(), 20.0, epsilon = 0.001);
        assert_relative_eq!(pre_fx.mix.get::<percent>(), 30.0, epsilon = 0.001);

        let post_fx = &effect.lanes[5];
        assert_relative_eq!(post_fx.gain.db(), 10.0, epsilon = 0.001);
        assert_relative_eq!(post_fx.pan.get::<percent>(), 20.0, epsilon = 0.001);
        assert_relative_eq!(post_fx.mix.get::<percent>(), 30.0, epsilon = 0.001);

        for lane in &effect.lanes[0..5] {
            assert_relative_eq!(lane.gain.db(), 10.0, epsilon = 0.001);
            assert_relative_eq!(lane.pan.get::<percent>(), 20.0, epsilon = 0.001);
            assert_relative_eq!(lane.mix.get::<percent>(), 30.0, epsilon = 0.001);
            assert_relative_eq!(lane.post.get::<percent>(), 40.0, epsilon = 0.001);
        }
    }

    // #[test]
    pub fn _macros_value_and_name() {
        let preset = read_effect_preset(
            "multipass",
            "multipass-macros-value_and_name-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_multipass().unwrap();
        assert_eq!(effect.macro_controls[1].name, "Macro Name 1");
        assert_relative_eq!(effect.macro_controls[0].value, 0.1);
        assert_eq!(effect.macro_controls[1].name, "Macro Name 2");
        assert_relative_eq!(effect.macro_controls[1].value, 0.2);
        assert_eq!(effect.macro_controls[2].name, "Macro Name 3");
        assert_relative_eq!(effect.macro_controls[2].value, 0.3);
        assert_eq!(effect.macro_controls[3].name, "Macro Name 4");
        assert_relative_eq!(effect.macro_controls[3].value, 0.4);
        assert_eq!(effect.macro_controls[4].name, "Macro Name 5");
        assert_relative_eq!(effect.macro_controls[4].value, 0.5);
        assert_eq!(effect.macro_controls[5].name, "Macro Name 6");
        assert_relative_eq!(effect.macro_controls[5].value, 0.6);
        assert_eq!(effect.macro_controls[6].name, "Macro Name 7");
        assert_relative_eq!(effect.macro_controls[6].value, 0.7);
        assert_eq!(effect.macro_controls[7].name, "Macro Name 8");
        assert_relative_eq!(effect.macro_controls[7].value, 0.8);
    }

    #[test]
    pub fn metadata() {
        let preset =
            read_effect_preset("multipass", "multipass-metadata-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Name");
        assert!(snapin.preset_path.is_empty());
        let metadata = &snapin.metadata;
        assert_eq!(metadata.author, Some("softdev.ca".to_owned()));
        assert_eq!(metadata.description, Some("Description".to_owned()));
    }

    // #[test]
    pub fn _parts() {
        let preset = read_effect_preset(
            "multipass",
            "multipass-split_2_100-split_3_2000-disabled-1.8.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_multipass().unwrap();
        assert_relative_eq!(effect.macro_controls[0].value, 0.1);
        // FIXME: Frequency splits
    }

    // #[test]
    pub fn _parts_version_2() {
        let preset = read_effect_preset(
            "multipass",
            "multipass-gain10-mix50-disabled-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_multipass().unwrap();
        assert_eq!(effect.gain.db(), 10.0);
        assert_eq!(effect.mix.get::<percent>(), 50.0);

        let preset = read_effect_preset(
            "multipass",
            "multipass-sideband-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_multipass().unwrap();
        assert_eq!(effect.external_input_mode, ExternalInputMode::Sideband);
    }
}

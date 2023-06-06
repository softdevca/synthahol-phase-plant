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

use std::any::{Any, type_name};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::EnumIter;
use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::{Decibels, MacroControl};

use super::{Effect, EffectMode};
use super::super::io::*;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Multipass {
    pub gain: Decibels,
    pub pan: Ratio,
    pub mix: Ratio,
    pub external_input_mode: ExternalInputMode,
    pub macro_controls: [MacroControl; MacroControl::COUNT],
    unknown1: [u8; 1957],
    unknown2: [u8; 10239],
    unknown3: [u8; 128],
}

impl Default for Multipass {
    fn default() -> Self {
        Self {
            gain: Decibels::ZERO,
            pan: Ratio::new::<percent>(50.0),
            mix: Ratio::new::<percent>(100.0),
            external_input_mode: ExternalInputMode::Off,
            macro_controls: MacroControl::defaults(),
            unknown1: [0_u8; 1957],
            unknown2: [0_u8; 10239],
            unknown3: [0_u8; 128],
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

        let mut effect = Multipass::default();
        let minimized = false;
        let enabled = true;

        reader.read_exact(&mut effect.unknown1)?;

        // FIXME: MUST SAVE THESE
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

        Ok(EffectReadReturn::new(Box::new(effect), enabled, minimized))
    }
}

impl EffectWrite for Multipass {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        _enabled: bool,
        _minimized: bool,
    ) -> io::Result<()> {
        writer.write_all_u8(&self.unknown1)?;
        writer.write_all_u8(&self.unknown2)?;
        writer.write_all_u8(&self.unknown3)
    }

    fn write_version(&self) -> u32 {
        1058
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
        assert_eq!(effect.macro_controls[0].value, 0.0);
        assert_eq!(effect.macro_controls[1].value, 0.0);
        assert_eq!(effect.macro_controls[2].value, 0.0);
        assert_eq!(effect.macro_controls[3].value, 0.0);
        assert_eq!(effect.macro_controls[4].value, 0.0);
        assert_eq!(effect.macro_controls[5].value, 0.0);
        assert_eq!(effect.macro_controls[6].value, 0.0);
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
            "multipass-1.8.5.phaseplant",
            "multipass-2.0.12.phaseplant",
            "multipass-2.0.16.phaseplant",
            "multipass-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("multipass", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let _effect = snapin.effect.as_multipass().unwrap();
            // assert_eq!(effect, &Default::default());
        }
    }

    // #[test]
    pub fn _parts_version_1() {
        let preset = read_effect_preset(
            "multipass",
            "multipass-macro_values-minimized-1.8.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_multipass().unwrap();
        assert_relative_eq!(effect.macro_controls[0].value, 0.1);
        assert_relative_eq!(effect.macro_controls[1].value, 0.2);
        assert_relative_eq!(effect.macro_controls[2].value, 0.3);
        assert_relative_eq!(effect.macro_controls[3].value, 0.4);
        assert_relative_eq!(effect.macro_controls[4].value, 0.5);
        assert_relative_eq!(effect.macro_controls[5].value, 0.6);
        assert_relative_eq!(effect.macro_controls[6].value, 0.7);
        assert_relative_eq!(effect.macro_controls[7].value, 0.8);

        let preset = read_effect_preset(
            "multipass",
            "multipass-pre_fx-gain5-pan25-mix50-1.8.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_multipass().unwrap();
        assert_relative_eq!(effect.gain.db(), 0.0);
        assert_relative_eq!(effect.pan.get::<percent>(), 25.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 25.0);

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

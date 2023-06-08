//! Group allows a collection of snapins to be minimized and disabled together.
//!
//! Group was added in Phase Plant version 2. It always have a host version
//! of 0.0.0.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 2.0.0               | 1007           |
//! | 2.0.16              | 1007           |

use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::SnapinId;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub name: Option<String>,
}

impl dyn Effect {
    #[must_use]
    pub fn as_group(&self) -> Option<&Group> {
        self.downcast_ref::<Group>()
    }
}

impl Effect for Group {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Group
    }
}

impl EffectRead for Group {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1007 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Group effect version {effect_version}"),
            ));
        }

        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "group_unknown_1")?;
        reader.expect_u32(0, "group_unknown_2")?;

        let group_id = reader.read_snapin_position()?;
        let name = reader.read_string_and_length()?;

        Ok(EffectReadReturn::new(
            Box::new(Group { name }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Group {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // group_unknown_1
        writer.write_u32(0)?; // group_unknown_2

        writer.write_snapin_id(group_id)?;
        writer.write_string_and_length_opt(&self.name)
    }

    fn write_version(&self) -> u32 {
        1007
    }
}

#[cfg(test)]
mod test {
    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Group::default();
        assert_eq!(None, effect.name);
    }

    #[test]
    pub fn disabled() {
        for file in &[
            "group-disabled-2.0.0.phaseplant",
            "group-disabled-2.0.12.phaseplant",
        ] {
            let preset = read_effect_preset("group", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(!snapin.enabled);
            assert!(!snapin.minimized);
        }
    }

    #[test]
    fn eq() {
        let effect = Group::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Group::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    pub fn init() {
        for file in &[
            "group-2.0.0.phaseplant",
            "group-2.0.12.phaseplant",
            "group-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("group", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_group().unwrap();
            assert_eq!(&Group::default(), effect);
        }
    }

    #[test]
    pub fn name() {
        let preset = read_effect_preset("group", "group-name-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(
            snapin.effect.as_group().unwrap().name,
            Some("New Name".to_owned())
        );

        let preset = read_effect_preset("group", "group-3_groups-name-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(
            snapin.effect.as_group().unwrap().name,
            Some("New Group 1".to_owned())
        );
        let snapin = &preset.lanes[0].snapins[1];
        assert_eq!(
            snapin.effect.as_group().unwrap().name,
            Some("New Group 2".to_owned())
        );
        let snapin = &preset.lanes[0].snapins[2];
        assert_eq!(
            snapin.effect.as_group().unwrap().name,
            Some("New Group 3".to_owned())
        );
    }

    #[test]
    pub fn multiple() {
        let preset = read_effect_preset("group", "group-3_groups-2.0.12.phaseplant").unwrap();
        let snapins = &preset.lanes[0].snapins;
        assert_eq!(snapins.len(), 3);
        assert!(snapins[0].effect.as_group().is_some());
        assert!(snapins[1].effect.as_group().is_some());
        assert!(snapins[2].effect.as_group().is_some());
    }
}

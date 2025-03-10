//! [Snap Heap](https://kilohearts.com/products/snap_heap) is a Snapin host.
//!
//! The ability to add Snap Heap as an effect was added in Phase Plant 1.8.0.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.0 to 1.8.5      | 1038           |
//! | 2.0.12              | 1050           |
//! | 2.0.16 to 2.1.0     | 1051           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::effect::EffectVersion;
use crate::effect::multipass::ExternalInputMode;
use crate::{Decibels, MacroControl, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct SnapHeap {
    pub gain: Decibels,
    pub mix: Ratio,
    pub external_input_mode: ExternalInputMode,
    pub macro_controls: [MacroControl; MacroControl::COUNT],
}

impl SnapHeap {
    pub fn default_version() -> EffectVersion {
        1051
    }
}

impl Default for SnapHeap {
    fn default() -> Self {
        Self {
            gain: Decibels::ZERO,
            mix: Ratio::new::<percent>(100.0),
            external_input_mode: ExternalInputMode::Off,
            macro_controls: MacroControl::defaults(),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_snap_heap(&self) -> Option<&SnapHeap> {
        self.downcast_ref::<SnapHeap>()
    }
}

impl Effect for SnapHeap {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::SnapHeap
    }
}

impl EffectRead for SnapHeap {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1038 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let effect = SnapHeap::default();
        let enabled = true;
        let minimized = false;
        let group_id = None;

        reader.skip(1849)?;

        if effect_version >= 1050 {
            reader.skip(10347)?;
        }
        if effect_version >= 1051 {
            // See Multipass for this 128 block, might be the same
            reader.skip(128)?;
        }

        Ok(EffectReadReturn::new(
            Box::new(effect),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for SnapHeap {
    fn write<W: Write + Seek>(
        &self,
        _writer: &mut PhasePlantWriter<W>,
        _snapin: &Snapin,
    ) -> io::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;

    use crate::effect::multipass::ExternalInputMode;
    use crate::effect::*;
    use crate::test::read_effect_preset;

    use super::SnapHeap;

    #[test]
    fn default() {
        let effect = SnapHeap::default();
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
        let effect = SnapHeap::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, SnapHeap::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "snap_heap-1.8.0.phaseplant",
            "snap_heap-1.8.5.phaseplant",
            "snap_heap-2.0.12.phaseplant",
            "snap_heap-2.0.16.phaseplant",
            "snap_heap-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("snap_heap", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_snap_heap().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    // #[test]
    fn _parts_version_1() {
        let preset =
            read_effect_preset("snap_heap", "snap_heap-disabled-1.8.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);

        let preset =
            read_effect_preset("snap_heap", "snap_heap-minimized-1.8.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);

        let preset =
            read_effect_preset("snap_heap", "snap_heap-sideband-1.8.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_snap_heap().unwrap();
        assert_eq!(effect.external_input_mode, ExternalInputMode::Sideband);
    }

    // #[test]
    fn _parts_version_2() {
        let preset = read_effect_preset(
            "snap_heap",
            "snap_heap-gain5-mix25-disabled-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_snap_heap().unwrap();
        assert_relative_eq!(effect.gain.db(), 5.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 25.0);

        let preset = read_effect_preset(
            "snap_heap",
            "snap_heap-macro_values-minimized-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_snap_heap().unwrap();
        assert_relative_eq!(effect.macro_controls[0].value, 0.1);
        assert_relative_eq!(effect.macro_controls[1].value, 0.2);
        assert_relative_eq!(effect.macro_controls[2].value, 0.3);
        assert_relative_eq!(effect.macro_controls[3].value, 0.4);
        assert_relative_eq!(effect.macro_controls[4].value, 0.5);
        assert_relative_eq!(effect.macro_controls[5].value, 0.6);
        assert_relative_eq!(effect.macro_controls[6].value, 0.7);
        assert_relative_eq!(effect.macro_controls[7].value, 0.8);
    }
}

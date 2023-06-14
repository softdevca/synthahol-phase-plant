//! [Transient Shaper](https://kilohearts.com/products/transient_shaper) is an
//! effect that modifies the attack and sustain of a sound.

//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.13     | 1027           |
//! | 2.0.16              | 1037           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::effect::{EffectVersion, SidechainMode};
use crate::Snapin;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct TransientShaper {
    pub attack: Ratio,
    pub pump: Ratio,
    pub sustain: Ratio,
    pub speed: Ratio,
    pub clip: bool,
    pub sidechain_mode: SidechainMode,
}

impl TransientShaper {
    pub fn default_version() -> EffectVersion {
        1037
    }
}

impl Default for TransientShaper {
    fn default() -> Self {
        Self {
            attack: Ratio::zero(),
            pump: Ratio::zero(),
            sustain: Ratio::zero(),
            speed: Ratio::new::<percent>(100.0),
            clip: false,
            sidechain_mode: SidechainMode::Off,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_transient_shaper(&self) -> Option<&TransientShaper> {
        self.downcast_ref::<TransientShaper>()
    }
}

impl Effect for TransientShaper {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::TransientShaper
    }
}

impl EffectRead for TransientShaper {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1027 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let attack = reader.read_ratio()?;
        let pump = reader.read_ratio()?;
        let sustain = reader.read_ratio()?;
        let speed = reader.read_ratio()?;
        let clip = reader.read_bool32()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "transient_shaper_unknown_1")?;
        reader.expect_u32(0, "transient_shaper_unknown_2")?;

        let group_id = if effect_version >= 1034 {
            reader.read_snapin_position()?
        } else {
            None
        };

        let sidechain_id = reader.read_u32()?;
        let sidechain_mode_str = reader.read_string_and_length()?;
        let sidechain_mode = SidechainMode::from_name(&sidechain_mode_str.unwrap_or_default())?;
        if sidechain_mode as u32 != sidechain_id {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Sidechain ID {sidechain_id:#x} does not match mode {sidechain_mode}"),
            ));
        }

        Ok(EffectReadReturn::new(
            Box::new(TransientShaper {
                attack,
                pump,
                sustain,
                speed,
                clip,
                sidechain_mode,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for TransientShaper {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_ratio(self.attack)?;
        writer.write_ratio(self.pump)?;
        writer.write_ratio(self.sustain)?;
        writer.write_ratio(self.speed)?;
        writer.write_bool32(self.clip)?;
        writer.write_bool32(snapin.enabled)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?; // transient_shaper_unknown_1
        writer.write_u32(0)?; // transient_shaper_unknown_2

        if snapin.effect_version >= 1034 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        writer.write_u32(self.sidechain_mode as u32)?;
        writer.write_string_and_length(self.sidechain_mode.to_string())
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
        let effect = TransientShaper::default();
        assert_eq!(effect.attack.get::<percent>(), 0.0);
        assert_eq!(effect.pump.get::<percent>(), 0.0);
        assert_eq!(effect.sustain.get::<percent>(), 0.0);
        assert_eq!(effect.speed.get::<percent>(), 100.0);
        assert!(!effect.clip);
        assert_eq!(effect.sidechain_mode, SidechainMode::Off);
    }

    #[test]
    fn eq() {
        let effect = TransientShaper::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, TransientShaper::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "transient_shaper-2.0.12.phaseplant",
            "transient_shaper-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("transient_shaper", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_transient_shaper().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset(
            "transient_shaper",
            "transient_shaper-atk10-pump20-sus30-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_transient_shaper().unwrap();
        assert_relative_eq!(effect.attack.get::<percent>(), 10.0, epsilon = 0.0001);
        assert_relative_eq!(effect.pump.get::<percent>(), 20.0, epsilon = 0.0001);
        assert_relative_eq!(effect.sustain.get::<percent>(), 30.0, epsilon = 0.0001);

        let preset = read_effect_preset(
            "transient_shaper",
            "transient_shaper-clip-atk-100-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_transient_shaper().unwrap();
        assert!(effect.clip);
        assert_relative_eq!(effect.attack.get::<percent>(), -100.0, epsilon = 0.01);

        let preset = read_effect_preset(
            "transient_shaper",
            "transient_shaper-sideband-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_transient_shaper().unwrap();
        assert_eq!(effect.sidechain_mode, SidechainMode::Sideband);

        let preset = read_effect_preset(
            "transient_shaper",
            "transient_shaper-speed500-disabled-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_transient_shaper().unwrap();
        assert_relative_eq!(effect.speed.get::<percent>(), 500.0);
    }
}

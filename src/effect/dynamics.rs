//! [Dynamics](https://kilohearts.com/products/dynamics) is a dynamics
//! processing effect that does upward and downward compression and expansion.
//!
//! The Dynamics effect was to Phase Plant in version 1.8.3.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1003           |
//! | 1.8.14              | 1003           |
//! | 2.0.16              | 1014           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::effect::EffectVersion;
use crate::{Decibels, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamics {
    pub attack: Ratio,
    pub release: Ratio,
    pub knee: Decibels,
    pub in_gain: Decibels,
    pub out_gain: Decibels,
    pub mix: Ratio,
    pub low_threshold: Decibels,
    pub high_threshold: Decibels,

    /// Ratios are `1.0 / value`
    pub low_ratio: f32,
    pub high_ratio: f32,
}

impl Dynamics {
    pub fn default_version() -> EffectVersion {
        1014
    }
}

impl Default for Dynamics {
    fn default() -> Self {
        Self {
            attack: Ratio::new::<percent>(100.0),
            release: Ratio::new::<percent>(100.0),
            knee: Decibels::new(2.5),
            in_gain: Decibels::ZERO,
            out_gain: Decibels::ZERO,
            mix: Ratio::new::<percent>(100.0),
            low_threshold: Decibels::new(-30.0),
            high_threshold: Decibels::new(-20.0),
            low_ratio: 1.0,
            high_ratio: 1.0,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_dynamics(&self) -> Option<&Dynamics> {
        self.downcast_ref::<Dynamics>()
    }
}

impl Effect for Dynamics {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Dynamics
    }
}

impl EffectRead for Dynamics {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1003 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let in_gain = reader.read_decibels_db()?;
        let out_gain = reader.read_decibels_db()?;
        let low_threshold = reader.read_decibels_db()?;
        let low_ratio = reader.read_f32()?;
        let high_threshold = reader.read_decibels_db()?;
        let high_ratio = reader.read_f32()?;
        let release = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "dynamics_unknown1")?;
        reader.expect_u32(0, "dynamics_unknown2")?;

        let attack = reader.read_ratio()?;
        let knee = reader.read_decibels_db()?;

        let group_id = if effect_version > 1003 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(Dynamics {
                attack,
                release,
                knee,
                in_gain,
                out_gain,
                mix,
                low_threshold,
                high_threshold,
                low_ratio,
                high_ratio,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Dynamics {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_f32(self.in_gain.db())?;
        writer.write_f32(self.out_gain.db())?;
        writer.write_f32(self.low_threshold.db())?;
        writer.write_f32(self.low_ratio)?;
        writer.write_f32(self.high_threshold.db())?;
        writer.write_f32(self.high_ratio)?;
        writer.write_ratio(self.release)?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(snapin.enabled)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?; // dynamics_unknown_1
        writer.write_u32(0)?; // dynamics_unknown_2

        writer.write_ratio(self.attack)?;
        writer.write_f32(self.knee.db())?;

        if snapin.effect_version > 1003 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::Decibels;
    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Dynamics::default();
        assert_eq!(effect.attack.get::<percent>(), 100.0);
        assert_eq!(effect.release.get::<percent>(), 100.0);
        assert_eq!(effect.knee, Decibels::new(2.5));
        assert_eq!(effect.in_gain, Decibels::ZERO);
        assert_eq!(effect.out_gain, Decibels::ZERO);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.low_threshold, Decibels::new(-30.0));
        assert_eq!(effect.high_threshold, Decibels::new(-20.0));
        assert_eq!(effect.low_ratio, 1.0);
        assert_eq!(effect.high_ratio, 1.0);
    }

    #[test]
    fn eq() {
        let effect = Dynamics::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Dynamics::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["dynamics-1.8.13.phaseplant", "dynamics-2.0.16.phaseplant"] {
            let preset = read_effect_preset("dynamics", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_dynamics().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "dynamics",
            "dynamics-low_thresh-50-high_thresh--5-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dynamics().unwrap();
        assert_eq!(effect.low_threshold, Decibels::new(-50.0));
        assert_eq!(effect.high_threshold, Decibels::new(-5.0));

        let preset = read_effect_preset(
            "dynamics",
            "dynamics-low_ratio2-high_ratio3-minimized-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_dynamics().unwrap();
        assert_relative_eq!(effect.low_ratio, 1.0 / 2.0);
        assert_relative_eq!(effect.high_ratio, 1.0 / 3.0);

        let preset = read_effect_preset(
            "dynamics",
            "dynamics-attack25-release50-knee10-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_dynamics().unwrap();
        assert_relative_eq!(effect.attack.get::<percent>(), 25.0);
        assert_relative_eq!(effect.release.get::<percent>(), 50.0, epsilon = 0.0001);
        assert_eq!(effect.knee, Decibels::new(10.0));

        let preset = read_effect_preset(
            "dynamics",
            "dynamics-in5-out10-mix20-disabled-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_dynamics().unwrap();
        assert_eq!(effect.in_gain, Decibels::new(5.0));
        assert_eq!(effect.out_gain, Decibels::new(10.0));
        assert_eq!(effect.mix.get::<percent>(), 20.0);

        let preset = read_effect_preset("dynamics", "dynamics-smacker-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Smacker");
        assert_eq!(snapin.preset_path, vec!["factory", "Smacker.ksot"]);
        assert!(!snapin.preset_edited);
    }
}

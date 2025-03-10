//! [Gate](https://kilohearts.com/products/gate) is a noise gate.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.13 to 1.8.20    | 1029           |
//! | 2.0.16              | 1040           |

use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Time;
use uom::si::time::{millisecond, second};

use crate::effect::{EffectVersion, SidechainMode};
use crate::{Decibels, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Gate {
    pub threshold: Decibels,

    // TODO: Store the range as Decibels once the normalization is figured out.
    // These are some example expected dB values for selected linear values of range.
    // dB, linear
    // 0.0, 0.0 // 0.5, 0.055841412
    // 4.0, 0.36927363
    // 50.0, 0.99683774
    // 75.0, 0.99982214
    // 25.0, 0.9437659
    // 100.0, 0.99999
    /// Linear range from 0.0..=100.0 dB where 100.0 is presented as infinity.
    pub range: f32,

    pub tolerance: Decibels,
    pub hold: Time,
    pub attack: Time,
    pub release: Time,
    pub look_ahead: bool,
    pub flip: bool,
    pub sidechain_mode: SidechainMode,
}

impl Gate {
    pub fn default_version() -> EffectVersion {
        1040
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_gate(&self) -> Option<&Gate> {
        self.downcast_ref::<Gate>()
    }
}

impl Gate {
    const DEFAULT_TOLERANCE_DB: f64 = 6.020599913279624;
    const DEFAULT_THRESHOLD_DB: f64 = -30.00000046767787;
}

impl Default for Gate {
    fn default() -> Self {
        Self {
            threshold: Decibels::new(Self::DEFAULT_THRESHOLD_DB as f32),
            range: 1.0,
            tolerance: Decibels::new(Self::DEFAULT_TOLERANCE_DB as f32),
            hold: Time::new::<millisecond>(25.0),
            attack: Time::new::<millisecond>(5.0),
            release: Time::new::<millisecond>(25.0),
            look_ahead: true,
            flip: false,
            sidechain_mode: SidechainMode::Off,
        }
    }
}

impl Effect for Gate {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Gate
    }
}

impl EffectRead for Gate {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1029 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Gate effect version {effect_version}"),
            ));
        }

        let attack = reader.read_seconds()?;
        let hold = reader.read_seconds()?;
        let release = reader.read_seconds()?;
        let threshold = reader.read_decibels_linear()?;
        let tolerance = reader.read_decibels_linear()?;
        let range = reader.read_f32()?;
        let look_ahead = reader.read_bool32()?; //?
        let flip = reader.read_bool32()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "gate_unknown1")?;
        reader.expect_u32(0, "gate_unknown2")?;

        let group_id = if effect_version > 1029 {
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
            Box::new(Gate {
                threshold,
                range,
                tolerance,
                hold,
                attack,
                release,
                look_ahead,
                flip,
                sidechain_mode,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Gate {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_f32(self.attack.get::<second>())?;
        writer.write_f32(self.hold.get::<second>())?;
        writer.write_f32(self.release.get::<second>())?;
        writer.write_f32(self.threshold.linear())?;
        writer.write_f32(self.tolerance.linear())?;
        writer.write_f32(self.range)?;

        writer.write_bool32(self.look_ahead)?;
        writer.write_bool32(snapin.enabled)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?; // gate_unknown1
        writer.write_u32(0)?; // gate_unknown2

        if snapin.effect_version > 1029 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        writer.write_u32(self.sidechain_mode as u32)?;
        writer.write_string_and_length(self.sidechain_mode.to_string())
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
        let effect = Gate::default();
        assert_relative_eq!(effect.threshold.db(), Gate::DEFAULT_THRESHOLD_DB as f32);
        assert_relative_eq!(effect.tolerance.db(), Gate::DEFAULT_TOLERANCE_DB as f32);
        assert_relative_eq!(effect.hold.get::<second>(), 0.025);
        assert_relative_eq!(effect.attack.get::<second>(), 0.005);
        assert_relative_eq!(effect.release.get::<second>(), 0.025);
        assert_eq!(effect.range, 1.0);
        assert!(effect.look_ahead);
        assert!(!effect.flip);
        assert_eq!(effect.sidechain_mode, SidechainMode::Off);
    }

    #[test]
    pub fn disabled() {
        let preset = read_effect_preset("gate", "gate-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Gate::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Gate::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    pub fn init() {
        for file in &[
            "gate-1.8.14.phaseplant",
            "gate-1.8.20.phaseplant",
            "gate-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("gate", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_gate().unwrap();
            assert_relative_eq!(effect.threshold.db(), Gate::DEFAULT_THRESHOLD_DB as f32);
            assert_relative_eq!(effect.tolerance.db(), Gate::DEFAULT_TOLERANCE_DB as f32);
            assert_relative_eq!(effect.hold.get::<second>(), 0.025);
            assert_relative_eq!(effect.attack.get::<second>(), 0.005);
            assert_relative_eq!(effect.release.get::<second>(), 0.025);
            assert_eq!(effect.range, 1.0);
            assert!(effect.look_ahead);
            assert!(!effect.flip);
            assert_eq!(effect.sidechain_mode, SidechainMode::Off);
        }
    }

    #[test]
    pub fn minimized() {
        let preset = read_effect_preset("gate", "gate-minimized-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    pub fn parts_version_1() {
        let preset =
            read_effect_preset("gate", "gate-attack2-release50-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_gate().unwrap();
        assert_relative_eq!(effect.attack.get::<second>(), 0.002);
        assert_relative_eq!(effect.release.get::<second>(), 0.05);

        let preset =
            read_effect_preset("gate", "gate-sideband-no_lookahead-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_gate().unwrap();
        assert!(!effect.look_ahead);
        assert_eq!(effect.sidechain_mode, SidechainMode::Sideband);

        let preset =
            read_effect_preset("gate", "gate-tol3-hold15-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_gate().unwrap();
        assert_relative_eq!(effect.tolerance.db(), 3.0, epsilon = 0.001);
        assert_relative_eq!(effect.hold.get::<second>(), 0.015, epsilon = 0.001);

        let preset =
            read_effect_preset("gate", "gate-thresh-15-range2-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_gate().unwrap();
        assert_eq!(effect.range, 0.20567179);
        assert_relative_eq!(effect.threshold.db(), -15.0, epsilon = 0.001);
    }

    #[test]
    pub fn parts_version_2() {
        let preset =
            read_effect_preset("gate", "gate-flip-no_lookahead-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_gate().unwrap();
        assert!(effect.flip);
        assert!(!effect.look_ahead);

        let preset = read_effect_preset("gate", "gate-range4-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_gate().unwrap();
        assert_relative_eq!(effect.range, 0.36927363);
    }
}

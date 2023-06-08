//! [Trance Gate](https://kilohearts.com/products/trance_gate) is a sequenced
//! gate.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.14              | 1038           |
//! | 2.0.16              | 1049           |

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::FromRepr;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::percent;
use uom::si::time::millisecond;

use crate::SnapinId;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Copy, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum PatternResolution {
    // The discriminants correspond to the file format.
    Eighth,
    EightTriplet,
    Sixteenth,
    SixteenthTriplet,
    ThirtySecond,
    ThirtySecondTriplet,
    SixtyFourth,
}

impl PatternResolution {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown pattern resolution mode {id}"),
            )
        })
    }
}

impl Display for PatternResolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PatternResolution::*;
        // Same nomenclature as Phase Plant.
        let msg = match self {
            Eighth => "1/8",
            EightTriplet => "1/8T",
            Sixteenth => "1/16",
            SixteenthTriplet => "1/16T",
            ThirtySecond => "1/32",
            ThirtySecondTriplet => "1/32T",
            SixtyFourth => "1/64",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranceGate {
    pub pattern_number: u32,

    /// How many steps are in the pattern. There are the full amount of
    /// [STEPS_MAX](Self::STEPS_MAX) steps for every pattern but only the first
    /// step count of steps are used.
    pub step_count: [usize; Self::PATTERN_COUNT],

    pub step_enabled: [[bool; Self::STEPS_MAX]; Self::PATTERN_COUNT],
    pub step_tied: [[bool; Self::STEPS_MAX]; Self::PATTERN_COUNT],
    pub attack: Time,
    pub decay: Time,
    pub sustain: Ratio,
    pub release: Time,
    pub resolution: PatternResolution,
    pub mix: Ratio,
}

impl TranceGate {
    pub const PATTERN_COUNT: usize = 8;

    /// Maximum number of steps in a pattern.
    pub const STEPS_MAX: usize = 64;

    pub(crate) const STEP_COUNT_DEFAULT: [usize; Self::PATTERN_COUNT] =
        [16, 16, 32, 16, 16, 16, 16, 64];
    pub(crate) const STEP_ENABLED_DEFAULT: [[bool; Self::STEPS_MAX]; Self::PATTERN_COUNT] = [
        [
            true, false, true, false, true, true, true, false, true, false, true, false, true,
            true, true, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, false, true, false, true, false, true, false, true, false, true, false, true,
            true, true, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, true, false, false, true, false, true, true, false, false, true, false, true,
            true, false, false, true, false, true, true, false, false, true, false, true, true,
            false, false, true, true, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false,
        ],
        [
            false, false, false, false, true, true, false, false, true, false, false, false, true,
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, false, true, false, true, false, true, false, true, true, false, false, true,
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, true, false, false, true, true, false, false, true, true, true, true, false,
            false, true, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, true, false, true, false, true, true, false, true, true, false, true, false,
            true, true, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, true, false, true, false, true, false, false, false, true, true, false, true,
            false, true, true, false, true, true, false, false, false, true, true, false, true,
            true, false, false, false, true, true, false, true, false, true, false, true, false,
            false, false, true, false, true, false, true, false, true, true, false, true, true,
            false, false, false, true, false, true, true, false, false, false, false, false,
        ],
    ];
    pub(crate) const STEP_TIED_DEFAULT: [[bool; Self::STEPS_MAX]; Self::PATTERN_COUNT] = [
        [
            false, false, false, false, true, true, false, false, false, false, false, false, true,
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            false, false, false, false, false, false, false, false, false, false, false, false,
            true, true, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false,
        ],
        [
            true, false, false, false, false, false, true, false, false, false, false, false, true,
            false, false, false, false, false, true, false, false, false, false, false, true,
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            false, false, false, false, true, false, false, false, false, false, false, false,
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false,
        ],
        [
            false, false, false, false, false, false, false, false, true, false, false, false,
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false,
        ],
        [
            true, false, false, false, true, false, false, false, true, true, true, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, false, false, false, false, true, false, false, true, false, false, false, false,
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false,
        ],
        [
            true, false, false, false, false, false, false, false, false, true, false, false,
            false, false, true, false, false, true, false, false, false, false, true, false, false,
            true, false, false, false, false, true, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, false, true, false,
            false, true, false, false, false, false, false, false, true, false, false, false,
            false, false, false,
        ],
    ];
}

impl Default for TranceGate {
    fn default() -> Self {
        Self {
            pattern_number: 1,
            step_count: TranceGate::STEP_COUNT_DEFAULT,
            step_enabled: TranceGate::STEP_ENABLED_DEFAULT,
            step_tied: TranceGate::STEP_TIED_DEFAULT,
            attack: Time::new::<millisecond>(13.2),
            decay: Time::new::<millisecond>(55.6),
            sustain: Ratio::new::<percent>(50.0),
            release: Time::new::<millisecond>(17.6),
            resolution: PatternResolution::ThirtySecond,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_trance_gate(&self) -> Option<&TranceGate> {
        self.downcast_ref::<TranceGate>()
    }
}

impl Effect for TranceGate {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::TranceGate
    }
}

impl EffectRead for TranceGate {
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

        let enabled = reader.read_bool32()?;
        let attack = reader.read_seconds()?;
        let decay = reader.read_seconds()?;
        let sustain = reader.read_ratio()?;
        let release = reader.read_seconds()?;
        let mix = reader.read_ratio()?;
        let resolution = PatternResolution::from_id(reader.read_u32()?)?;
        let pattern_number = reader.read_u32()?;

        let mut step_count = [0_usize; TranceGate::PATTERN_COUNT];
        let mut step_enabled = [[false; TranceGate::STEPS_MAX]; TranceGate::PATTERN_COUNT];
        let mut step_tied = [[false; TranceGate::STEPS_MAX]; TranceGate::PATTERN_COUNT];
        for pattern_index in 0..TranceGate::PATTERN_COUNT {
            step_count[pattern_index] = reader.read_u32()? as usize;
            for sequence_index in 0..TranceGate::STEPS_MAX {
                step_enabled[pattern_index][sequence_index] = reader.read_bool32()?;
                step_tied[pattern_index][sequence_index] = reader.read_bool32()?;
            }
        }

        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "trance_gate_unknown_1")?;
        reader.expect_u32(0, "trance_gate_unknown_2")?;

        let group_id = if effect_version > 1038 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(TranceGate {
                pattern_number,
                step_count,
                step_enabled,
                step_tied,
                attack,
                decay,
                sustain,
                release,
                resolution,
                mix,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for TranceGate {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_seconds(self.attack)?;
        writer.write_seconds(self.decay)?;
        writer.write_ratio(self.sustain)?;
        writer.write_seconds(self.release)?;
        writer.write_ratio(self.mix)?;
        writer.write_u32(self.resolution as u32)?;
        writer.write_u32(self.pattern_number)?;
        for pattern_index in 0..TranceGate::PATTERN_COUNT {
            writer.write_u32(self.step_count[pattern_index] as u32)?;
            for sequence_index in 0..TranceGate::STEPS_MAX {
                writer.write_bool32(self.step_enabled[pattern_index][sequence_index])?;
                writer.write_bool32(self.step_tied[pattern_index][sequence_index])?;
            }
        }

        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // trace_gate_unknown_1
        writer.write_u32(0)?; // trace_gate_unknown_2

        if self.write_version() > 1038 {
            writer.write_snapin_id(group_id)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1049
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::time::millisecond;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = TranceGate::default();
        assert_eq!(effect.pattern_number, 1);
        assert_eq!(effect.step_count, TranceGate::STEP_COUNT_DEFAULT);
        assert_eq!(effect.step_enabled, TranceGate::STEP_ENABLED_DEFAULT);
        assert_eq!(effect.step_tied, TranceGate::STEP_TIED_DEFAULT);
        assert_eq!(effect.attack.get::<millisecond>(), 13.2);
        assert_eq!(effect.decay.get::<millisecond>(), 55.6);
        assert_eq!(effect.sustain.get::<percent>(), 50.0);
        assert_eq!(effect.release.get::<millisecond>(), 17.6);
        assert_eq!(effect.resolution, PatternResolution::ThirtySecond);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("trance_gate", "trance_gate-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = TranceGate::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, TranceGate::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "trance_gate-1.8.14.phaseplant",
            "trance_gate-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("trance_gate", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_trance_gate().unwrap();
            assert_eq!(effect.pattern_number, 1);
            assert_eq!(effect.step_count, TranceGate::STEP_COUNT_DEFAULT);
            assert_eq!(effect.step_enabled, TranceGate::STEP_ENABLED_DEFAULT);
            assert_eq!(effect.step_tied, TranceGate::STEP_TIED_DEFAULT);
            assert_relative_eq!(effect.attack.get::<millisecond>(), 13.2, epsilon = 0.1);
            assert_relative_eq!(effect.decay.get::<millisecond>(), 55.6, epsilon = 0.1);
            assert_relative_eq!(effect.sustain.get::<percent>(), 50.0, epsilon = 0.001);
            assert_relative_eq!(effect.release.get::<millisecond>(), 17.6, epsilon = 0.1);
            assert_eq!(effect.resolution, PatternResolution::ThirtySecond);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("trance_gate", "trance_gate-minimized-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "trance_gate",
            "trance_gate-count11-sustain80-release25-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_trance_gate().unwrap();
        assert_eq!(effect.step_count[0], 11);
        assert_relative_eq!(effect.sustain.get::<percent>(), 80.0);
        assert_relative_eq!(effect.release.get::<millisecond>(), 25.0, epsilon = 0.001);

        let preset = read_effect_preset(
            "trance_gate",
            "trance_gate-eighth-mix66-disabled-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_trance_gate().unwrap();
        assert_eq!(effect.mix.get::<percent>(), 66.0);
        assert_eq!(effect.resolution, PatternResolution::Eighth);

        let preset = read_effect_preset(
            "trance_gate",
            "trance_gate-selected3-attack20-decay75-minimized-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_trance_gate().unwrap();
        assert_eq!(effect.pattern_number, 3);
        assert_relative_eq!(effect.attack.get::<millisecond>(), 20.0, epsilon = 0.001);
        assert_relative_eq!(effect.decay.get::<millisecond>(), 75.0, epsilon = 0.001);
    }

    #[test]
    fn parts_version_2() {
        let preset = read_effect_preset(
            "trance_gate",
            "trance_gate-eighth-selected7-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_trance_gate().unwrap();
        assert_eq!(effect.resolution, PatternResolution::Eighth);
        assert_eq!(effect.pattern_number, 7);

        let preset = read_effect_preset(
            "trance_gate",
            "trance_gate-sixteenth-all_off-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_trance_gate().unwrap();
        assert_eq!(effect.resolution, PatternResolution::Sixteenth);
        assert!(effect
            .step_count
            .into_iter()
            .all(|c| c == TranceGate::STEPS_MAX));
        for pattern in effect.step_enabled {
            assert!(pattern.into_iter().all(|enabled| !enabled));
        }
        for pattern in effect.step_tied {
            assert!(pattern.into_iter().all(|tied| !tied));
        }

        // The first pattern has only one tied step. The second pattern has
        // all steps tied.
        let preset = read_effect_preset(
            "trance_gate",
            "trance_gate-sixteenth_triplet-all_on-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_trance_gate().unwrap();
        assert_eq!(effect.resolution, PatternResolution::SixteenthTriplet);
        assert!(effect
            .step_count
            .into_iter()
            .all(|c| c == TranceGate::STEPS_MAX));
        for pattern in effect.step_enabled {
            assert!(pattern.into_iter().all(|enabled| enabled));
        }
        assert!(effect.step_tied[0][..effect.step_tied[0].len() - 1]
            .iter()
            .all(|tied| *tied));
        assert!(effect.step_tied[1].into_iter().all(|tied| !tied));
    }
}

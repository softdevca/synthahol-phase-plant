//! Modulation routes control and audio signals.

use std::fmt::{Display, Formatter};

use strum_macros::Display;
use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use super::*;

/// How many total macro connections that link a control to a parameter.
pub const MODULATIONS_MAX: usize = 100;

type SourceId = u16;
type TargetId = u16;

type CategoryId = u16;
type ModuleId = u16;
type ParameterId = u16;

type LaneId = u8;

#[derive(Clone, Copy, Debug, Display, PartialEq)]
pub enum RateMode {
    Audio,
    Control,
}

impl RateMode {
    // Use audio rate modulation when the high bit is set on the source or
    // target ID.

    const MODE_MASK: u16 = 0x8000;
    const ID_MASK: u16 = !0x8000;

    /// Return a modified ID that includes the rate mode.
    #[must_use]
    fn add_id(&self, id: u16) -> u16 {
        match self {
            RateMode::Audio => id | Self::MODE_MASK,
            RateMode::Control => id & Self::ID_MASK,
        }
    }

    /// Remove the rate mode component and return it and the modified ID.
    fn split_id(id: u16) -> (RateMode, u16) {
        if id & Self::MODE_MASK != 0 {
            (RateMode::Audio, id & Self::ID_MASK)
        } else {
            (RateMode::Control, id)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Modulation {
    pub enabled: bool,
    pub source: ModulationSource,
    pub target: ModulationTarget,
    pub amount: Ratio,
    pub curve: Ratio,
}

impl Modulation {
    pub fn new(source: ModulationSource, target: ModulationTarget, amount: Ratio) -> Self {
        Self {
            source,
            target,
            amount,
            ..Default::default()
        }
    }
}

impl Default for Modulation {
    fn default() -> Self {
        Self {
            enabled: true,
            source: Default::default(),
            target: Default::default(),
            amount: Ratio::zero(),
            curve: Ratio::zero(),
        }
    }
}

impl Display for Modulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let modulation_percent_fmt =
            Ratio::format_args(percent, uom::fmt::DisplayStyle::Abbreviation);
        let msg = format!(
            "{} → {} {}",
            self.source,
            self.target,
            modulation_percent_fmt.with(self.amount)
        );
        f.write_str(&msg)?;

        if !self.enabled {
            f.write_str(" (disabled)")?;
        }
        Ok(())
    }
}

//
// Source
//

#[derive(Clone, Debug, PartialEq)]
pub enum ModulationSource {
    AudioRate {
        module_id: ModuleId,
        parameter_id: ParameterId,
    },
    MacroControl(u8),
    Unknown {
        category_id: CategoryId,
        source_id: SourceId,

        /// Why it is unknown and not recognized.
        reason: Option<String>,
    },
}

impl Default for ModulationSource {
    fn default() -> Self {
        Self::Unknown {
            category_id: Self::LOCAL_CATEGORY_ID,
            source_id: 0,
            reason: None,
        }
    }
}

/// Shows human readable IDs as positions, starting at 1 instead of 0.
impl Display for ModulationSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ModulationSource::*;
        let msg = match self {
            AudioRate {
                module_id,
                parameter_id,
            } => format!("Audio rate module {module_id} parameter {parameter_id}"),
            // Generator { id, target } => format!(
            //     "Generator {:0x} {}",
            //     id + 1,
            //     target.to_string().to_ascii_lowercase()
            // ),
            MacroControl(id) => format!("Macro {}", id + 1),
            // ModulatorDepth(id) => format!("Modulator {} depth", id + 1),
            // Modulator { modulator_id, parameter_id } => format!("Modulator {}, parameter {}", modulator_id + 1, parameter_id + 1),
            // ModWheel => "Mod Wheel".to_owned(),
            // Snapin { position, target_id } => format!("Snapin {position}, target {target_id}"),
            Unknown {
                category_id,
                source_id,
                reason,
            } => {
                let msg = format!("Category {category_id:#x} source {source_id:#x}");
                if let Some(reason) = reason {
                    format!("{msg} ({reason})")
                } else {
                    msg
                }
            }
        };
        f.write_str(&msg)
    }
}

impl ModulationSource {
    const LOCAL_CATEGORY_ID: CategoryId = 0xFFFF;

    // In order of specifier.
    // const MACRO_CONTROL_START: u16 = 0;
    // const MACRO_CONTROL_END: u16 = 7;
    // const MODULATOR_DEPTH_START: u16 = 8;
    // const MODULATOR_DEPTH_END: u16 = 39;
    // const MOD_WHEEL: u16 = 40;

    /// The lower 16 bits of a modulation source ID are always 0xFFFF.
    pub fn id(&self) -> u32 {
        use ModulationSource::*;

        if let Unknown {
            category_id,
            source_id,
            reason: _,
        } = self
        {
            return ((*source_id as u32) << 16) | (*category_id as u32);
        }

        let source_id = match self {
            AudioRate {
                module_id,
                parameter_id,
            } => RateMode::Audio.add_id(module_id << 4 | *parameter_id),
            MacroControl(id) => RateMode::Control.add_id(*id as u16),
            Unknown { .. } => unreachable!(),
        };
        (source_id as u32) << 16 | ModulationSource::LOCAL_CATEGORY_ID as u32
    }
}

impl From<u32> for ModulationSource {
    fn from(id: u32) -> Self {
        use ModulationSource::*;

        let category_id = (id & 0xFFFF) as CategoryId;

        // Split the rate mode from the ID then the module and parameters
        // from that
        let source_id = (id >> 16) as SourceId;
        let (rate_mode, part_id) = RateMode::split_id(source_id);

        if category_id == Self::LOCAL_CATEGORY_ID {
            match rate_mode {
                RateMode::Audio => {
                    let module_id = part_id >> 4;
                    let parameter_id = (part_id & 0x000F) as ParameterId;
                    AudioRate {
                        module_id,
                        parameter_id,
                    }
                }
                RateMode::Control => match part_id {
                    0..=7 => MacroControl(part_id as u8),
                    _ => Unknown {
                        category_id,
                        source_id,
                        reason: Some(format!(
                            "Control rate source {part_id:#x} is not recognized"
                        )),
                    },
                },
            }
        } else {
            Unknown {
                category_id,
                source_id,
                reason: Some(format!("Unknown category {category_id:#x}")),
            }
        }
    }
}

//
// Target
//

#[derive(Clone, Debug, PartialEq)]
pub enum ModulationTarget {
    Host {
        parameter: HostParameter,
        rate_mode: RateMode,
    },
    Modulation {
        parameter_id: TargetId,
        rate_mode: RateMode,
    },
    Snapin {
        snapin_id: u16,
        parameter_id: TargetId,
        rate_mode: RateMode,
    },
    Unknown {
        category_id: CategoryId,
        parameter_id: TargetId,
        rate_mode: RateMode,
    },
}

impl Default for ModulationTarget {
    fn default() -> Self {
        Self::Unknown {
            category_id: Self::HOST_CATEGORY_ID,
            parameter_id: 0,
            rate_mode: RateMode::Control,
        }
    }
}

/// Shows human readable IDs as positions, starting at 1 instead of 0.
impl Display for ModulationTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ModulationTarget::*;
        let msg = match self {
            Host {
                parameter: target,
                rate_mode,
            } => {
                format!(
                    "Host {} target {target}",
                    rate_mode.to_string().to_lowercase()
                )
                // Lane { lane_id, parameter } => format!("Lane {} {}", lane_id + 1, parameter.to_string().to_lowercase()),
            }
            Modulation {
                parameter_id: target_id,
                rate_mode,
            } => format!(
                "Modulation {} target {target_id:#x}",
                rate_mode.to_string().to_lowercase()
            ),
            Snapin {
                snapin_id,
                parameter_id: target_id,
                rate_mode,
            } => format!(
                "Snapin {snapin_id:#x} {} target {target_id:#x}",
                rate_mode.to_string().to_lowercase()
            ),
            Unknown {
                category_id: module_id,
                parameter_id: target_id,
                rate_mode,
            } => format!(
                "Unknown module {module_id:#x} {} target {target_id:#x}",
                rate_mode.to_string().to_lowercase()
            ),
        };
        f.write_str(&msg)
    }
}

impl ModulationTarget {
    const HOST_CATEGORY_ID: CategoryId = 0xFFFF;
    const MODULATION_CATEGORY_ID: CategoryId = 0xFFFD;

    // Used internally by Phase Plant, should never be seen in a file.
    // Documented here in case it is encountered.
    const _PARENT_CATEGORY_ID: CategoryId = 0xFFFE;

    // TODO Might be +/- up to 3
    const LANE_START: SourceId = 0x019d;
    const LANE_SIZE: u16 = 5;
    const LANE_END: SourceId = Self::LANE_START + (Self::LANE_SIZE * crate::Lane::COUNT as u16) - 1;

    pub fn id(&self) -> u32 {
        use ModulationTarget::*;
        match self {
            Host {
                parameter: target,
                rate_mode,
            } => (rate_mode.add_id(target.id()) as u32) << 16 | Self::HOST_CATEGORY_ID as u32,
            Modulation {
                parameter_id: target_id,
                rate_mode,
            } => (rate_mode.add_id(*target_id) as u32) << 16 | Self::MODULATION_CATEGORY_ID as u32,
            Snapin {
                snapin_id,
                parameter_id: target_id,
                rate_mode,
            } => (rate_mode.add_id(*target_id) as u32) << 16 | *snapin_id as u32,
            Unknown {
                category_id: module_id,
                parameter_id: target_id,
                rate_mode,
            } => (rate_mode.add_id(*target_id) as u32) << 16 | *module_id as u32,
        }
    }
}

impl From<u32> for ModulationTarget {
    fn from(id: u32) -> Self {
        use ModulationTarget::*;

        let (rate_mode, target_id) = RateMode::split_id((id >> 16) as u16);

        let category_id = (id & 0xFFFF) as u16;
        if category_id == Self::HOST_CATEGORY_ID {
            use HostParameter::*;
            let parameter: HostParameter = match target_id {
                // Start with ranges
                Self::LANE_START..=Self::LANE_END => {
                    let lane_id = ((target_id - Self::LANE_START) / Self::LANE_SIZE) as LaneId;
                    let parameter_id = (target_id - Self::LANE_START) % Self::LANE_SIZE;
                    match parameter_id {
                        0 => LaneMix(lane_id),
                        1 => LaneGain(lane_id),
                        _ => Unknown {
                            target_id,
                            reason: Some(format!("Lane parameter {parameter_id} not recognized")),
                        },
                    }
                }

                // Individual parameters
                9711 => UnisonBias,
                3006 => UnisonBlend,
                3004 => UnisonDetune,
                3005 => UnisonSpread,
                _ => Unknown {
                    target_id,
                    reason: Some(format!("Host parameter {target_id} not recognized")),
                },
            };

            Host {
                parameter,
                rate_mode,
            }
        } else if category_id == Self::MODULATION_CATEGORY_ID {
            // match target_id {
            //     _ => Unknown {
            Unknown {
                category_id,
                parameter_id: target_id,
                rate_mode,
            }
            // }
        } else {
            Snapin {
                snapin_id: category_id,
                parameter_id: target_id,
                rate_mode,
            }
        }
    }
}

pub enum AudioRateTargetParameter {
    // The discriminants match the file format
    Frequency = 0,
    Pitch,
    Phase,
    RingMod,
    Cutoff,
    Q,
    Drive,
    Aux,
    Harmonic,
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum HostParameter {
    LaneGain(LaneId),
    LaneMix(LaneId),
    UnisonBias,
    UnisonBlend,
    UnisonDetune,
    UnisonSpread,
    Unknown {
        target_id: TargetId,
        reason: Option<String>,
    },
}

impl HostParameter {
    fn id(&self) -> TargetId {
        use HostParameter::*;
        match self {
            LaneGain(lane_id) => {
                ModulationTarget::LANE_START + (*lane_id as u16 * ModulationTarget::LANE_SIZE) + 1
            }
            LaneMix(lane_id) => {
                ModulationTarget::LANE_START + (*lane_id as u16 * ModulationTarget::LANE_SIZE)
            }
            UnisonBias => 9711,
            UnisonBlend => 3006,
            UnisonDetune => 3004,
            UnisonSpread => 3005,
            Unknown {
                target_id,
                reason: _,
            } => *target_id,
        }
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::num::Zero;
    use uom::si::f32::Ratio;
    use uom::si::ratio::{percent, ratio};

    use crate::modulation::HostParameter::{LaneGain, LaneMix};
    use crate::modulation::{HostParameter, ModulationSource, ModulationTarget, RateMode};
    use crate::modulator::ModulatorId;
    use crate::test::read_preset;

    /// Macro 1 goes to Lane 1 gain and mix, Macro 2 to Lane 2, Macro 3 to
    /// Lane 3.
    #[test]
    fn lane_gain_and_mix() {
        let preset = read_preset(
            "modulation",
            "macros-1to3_to_lanes_gain_and_mix-2.1.0.phaseplant",
        );
        assert_eq!(6, preset.modulations.len());
        for index in (0..6).step_by(2) {
            let modulation = &preset.modulations.get(index).unwrap();
            let mod_pos = (index / 2) as ModulatorId;
            assert_eq!(modulation.source, ModulationSource::MacroControl(mod_pos));
            assert_eq!(
                modulation.target,
                ModulationTarget::Host {
                    parameter: LaneMix(mod_pos),
                    rate_mode: RateMode::Control,
                }
            );
            let modulation = &preset.modulations.get(index + 1).unwrap();
            assert_eq!(modulation.source, ModulationSource::MacroControl(mod_pos));
            assert_eq!(
                modulation.target,
                ModulationTarget::Host {
                    parameter: LaneGain(mod_pos),
                    rate_mode: RateMode::Control,
                }
            );
        }
    }

    #[test]
    fn source_from() {
        use ModulationSource::*;
        assert_eq!(ModulationSource::from(0x0000FFFF), MacroControl(0));
        assert_eq!(ModulationSource::from(0x0002FFFF), MacroControl(2));
        assert!(matches!(
            ModulationSource::from(0x7234FFFF),
            Unknown {
                category_id: 0xFFFF,
                source_id: 0x7234,
                reason: _,
            }
        ));
        assert_eq!(
            ModulationSource::from(0x8234FFFF),
            AudioRate {
                module_id: 0x23,
                parameter_id: 0x4,
            },
        );
    }

    /// Converting from a source ID and back again must result in the same ID.
    #[test]
    fn source_id() {
        for id in 0..=0xFFFF {
            let id_with_module = id << 16 | 0xFFFF;
            assert_eq!(id_with_module, ModulationSource::from(id_with_module).id());
        }
    }

    #[test]
    fn target_from() {
        use ModulationTarget::*;
        assert!(matches!(
            ModulationTarget::from(0xF234FFFF),
            Host {
                parameter: HostParameter::Unknown {
                    target_id: 0x7234,
                    reason: _
                },
                rate_mode: RateMode::Audio,
            },
        ));
        assert_eq!(
            ModulationTarget::from(0x019DFFFF),
            Host {
                parameter: LaneMix(0),
                rate_mode: RateMode::Control,
            }
        );
    }

    /// Converting from a target ID and back again must result in the same ID.
    #[test]
    fn target_id() {
        for id in 0..=0xFFFF {
            let id_with_module = id << 16 | 0xFFFF;
            assert_eq!(id_with_module, ModulationTarget::from(id_with_module).id());
        }
    }

    /// Macro 3 to unison detune, spread, blend, bias.
    #[test]
    fn unison_target() {
        let preset = read_preset(
            "modulation",
            "macro-3-detune-spread-blend-bias-2.0.16.phaseplant",
        );

        assert_eq!(preset.modulations.len(), 4);
        for modulation in &preset.modulations {
            assert!(modulation.enabled);
            assert_eq!(modulation.curve, Ratio::zero());
            assert_eq!(modulation.source, ModulationSource::MacroControl(2));
        }

        let modulation = &preset.modulations[0];
        assert_relative_eq!(
            modulation.amount.get::<ratio>(),
            10.0 / 200.0,
            epsilon = 0.0001
        );
        assert_eq!(
            modulation.target,
            ModulationTarget::Host {
                parameter: HostParameter::UnisonDetune,
                rate_mode: RateMode::Control,
            }
        );

        let modulation = &preset.modulations[1];
        assert_relative_eq!(modulation.amount.get::<percent>(), 20.0, epsilon = 0.0001);
        assert_eq!(
            modulation.target,
            ModulationTarget::Host {
                parameter: HostParameter::UnisonSpread,
                rate_mode: RateMode::Control,
            }
        );

        let modulation = &preset.modulations[2];
        assert_relative_eq!(
            modulation.amount.get::<ratio>(),
            40.0 / 200.0,
            epsilon = 0.0001
        );
        assert_eq!(
            modulation.target,
            ModulationTarget::Host {
                parameter: HostParameter::UnisonBias,
                rate_mode: RateMode::Control,
            }
        );

        let modulation = &preset.modulations[3];
        assert_relative_eq!(modulation.amount.get::<percent>(), 30.0, epsilon = 0.0001);
        assert_eq!(
            modulation.target,
            ModulationTarget::Host {
                parameter: HostParameter::UnisonBlend,
                rate_mode: RateMode::Control,
            }
        );
    }
}

#[cfg(disabled)]
#[cfg(test)]
mod test {
    /// Mod wheel to various parts of two envelope output generators.
    #[test]
    fn mod_wheel_envelope_outputs() {
        let preset = read_preset("modulation", "mod_wheel-envelope_outputs-2.1.0.phaseplant");
        assert_eq!(preset.modulations.len(), 13);
        for modulation in &preset.modulations {
            assert!(modulation.enabled);
            assert_eq!(modulation.curve, Ratio::zero());
            assert_eq!(modulation.source, ModulationSource::ModWheel);
        }

        // First envelope
        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Attack,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::AttackCurve,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Decay,
            }
        );
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::DecayFalloff,
            }
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Sustain,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Release,
            }
        );
        assert_eq!(
            preset.modulations[6].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::ReleaseFalloff,
            }
        );
        assert_eq!(
            preset.modulations[7].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Delay,
            }
        );
        assert_eq!(
            preset.modulations[8].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Hold,
            }
        );
        assert_eq!(
            preset.modulations[9].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::OutputGain,
            }
        );
        assert_eq!(
            preset.modulations[10].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pan,
            }
        );

        // Second envelope only has attack and gain.
        assert!(matches!(
            preset.modulations[11].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::Attack
            }
        ));
        assert!(matches!(
            preset.modulations[12].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::OutputGain
            }
        ));
    }

    /// Mod wheel to three Note modulators.
    #[test]
    fn mod_wheel_note_modulators() {
        let preset = read_preset("modulation", "mod_wheel-modulator_notes-2.0.16.phaseplant");
        assert_eq!(preset.modulations.len(), 3);
        for modulation in &preset.modulations {
            assert!(modulation.enabled);
            assert_eq!(modulation.curve, Ratio::zero());
            assert_relative_eq!(modulation.amount.get::<percent>(), 0.0, epsilon = 0.0001);
            assert_eq!(modulation.source, ModulationSource::ModWheel);
        }

        let modulation = &preset.modulations[0];
        assert_eq!(modulation.amount.get::<percent>(), 0.0);
        assert_eq!(
            modulation.destination,
            ModulationTarget::Modulator {
                modulator_id: 0,
                parameter_id: 0,
            }
        );
        // FIXME: CHECK OTHERS
    }

    #[test]
    fn mod_wheel_version_1() {
        let preset = read_preset("modulation", "mod_wheel-glide_time-65-1.8.25.phaseplant");
        assert_eq!(1, preset.modulations.len());
        assert_relative_eq!(preset.mod_wheel_value.get::<percent>(), 1.6);
        let modulation = &preset.modulations.get(0).unwrap();
        assert!(modulation.enabled);
        assert_eq!(modulation.curve, Ratio::zero());
        assert_relative_eq!(
            modulation.amount.get::<percent>(),
            64.7999,
            epsilon = 0.0001
        );
        assert_eq!(modulation.source, ModulationSource::ModWheel);
        assert_eq!(modulation.destination, ModulationTarget::GlideTime);

        let preset = read_preset("modulation", "mod_wheel-master_gain-100-1.8.25.phaseplant");
        let modulation = &preset.modulations.get(0).unwrap();
        assert_relative_eq!(modulation.amount.get::<percent>(), 100.0);
        assert_eq!(modulation.source, ModulationSource::ModWheel);
        assert_eq!(modulation.destination, ModulationTarget::MasterGain);
    }

    #[test]
    fn mod_wheel_macros_version_1() {
        let preset = read_preset("modulation", "mod_wheel-all_macros-1.8.25.phaseplant");
        for (modulation_index, modulation) in preset.modulations.iter().enumerate() {
            assert!(modulation.enabled);
            assert_eq!(modulation.curve, Ratio::zero());
            assert_relative_eq!(modulation.amount.get::<percent>(), 0.0);
            assert_eq!(modulation.source, ModulationSource::ModWheel);
            assert_eq!(
                modulation.destination,
                ModulationTarget::MacroControl(modulation_index as u8)
            );
        }
        let preset = read_preset("modulation", "mod_wheel-macro1-50-1.8.25.phaseplant");
        assert_eq!(1, preset.modulations.len());
        let modulation = preset.modulations.get(0).unwrap();
        assert_relative_eq!(preset.mod_wheel_value.get::<percent>(), 1.6);
        assert!(modulation.enabled);
        assert_eq!(modulation.curve, Ratio::zero());
        assert_relative_eq!(modulation.amount.get::<percent>(), 50.0);
        assert_eq!(modulation.source, ModulationSource::ModWheel);
        assert_eq!(modulation.destination, ModulationTarget::MacroControl(0));

        let preset = read_preset("modulation", "mod_wheel-macro2--32-1.8.25.phaseplant");
        assert_relative_eq!(preset.mod_wheel_value.get::<percent>(), 1.6);
        let modulation = preset.modulations.get(0).unwrap();
        assert!(modulation.enabled);
        assert_eq!(modulation.curve, Ratio::zero());
        assert_relative_eq!(
            modulation.amount.get::<percent>(),
            -31.9999999,
            epsilon = 0.0001
        );
    }

    #[test]
    fn mod_wheel_macros_version_2() {
        let preset = read_preset("modulation", "mod_wheel-macro1-50-2.0.12.phaseplant");
        assert_eq!(1, preset.modulations.len());
        let modulation = preset.modulations.get(0).unwrap();
        assert!(modulation.enabled);
        assert_eq!(modulation.curve, Ratio::zero());
        assert_relative_eq!(modulation.amount.get::<percent>(), 50.0);
        assert_eq!(modulation.source, ModulationSource::ModWheel);
        assert_eq!(modulation.destination, ModulationTarget::MacroControl(0));
    }

    /// Modulator that modulates a modulation.
    // #[test]
    fn _modulate_modulation() {
        let preset = read_preset("modulation", "random-modulates-other-2.1.0.phaseplant");
        assert_eq!(preset.modulations.len(), 2);

        let random_to_analog = &preset.modulations[0];
        assert!(random_to_analog.enabled);
        assert!(matches!(
            random_to_analog.source,
            ModulationSource::ModulatorDepth(0)
        ));
        // FIXME: DEST
        // assert!(matches!(random_to_analog.destination, ModulationDest::Generator { generator_id: 0, parameter_id: 0 }));

        let random_to_modulation = &preset.modulations[1];
        assert!(random_to_modulation.enabled);
        assert!(matches!(
            random_to_modulation.source,
            ModulationSource::ModulatorDepth(1)
        ));
        // FIXME: OTHER MODULATOR
        // assert!(matches!(modulation.destination, ModulationDest::Generator { generator_id: 0, parameter_id: 0 }));
    }

    #[test]
    fn note_to_analog_oscillator() {
        let preset = read_preset("modulation", "note-to-analog_oscillator-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Sync,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::PulseWidth,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Level,
            }
        );
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pitch,
            }
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Harmonic,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Frequency,
            }
        );
        assert_eq!(
            preset.modulations[6].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::PhaseOffset,
            }
        );
    }

    // #[test]
    fn _note_to_curve_outputs() {
        let preset = read_preset("modulation", "note-to-curve_outputs-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        // First generator
        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::CurveOutputRate(1),
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::OutputGain,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pan,
            }
        );

        // Second generator
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::CurveOutputRate(2),
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::OutputGain,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::Pan,
            }
        );
    }

    // #[test]
    fn _note_to_distortion_effects() {
        let preset = read_preset("modulation", "note-to-distortion_effects-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        // First generator
        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Drive,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Bias,
            }
        );
        assert_eq!(preset.modulations[2].destination, DistortionEffectSpread(1),);
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Mix,
            }
        );

        // Second generator
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::Drive,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::Bias,
            }
        );
        assert_eq!(preset.modulations[6].destination, DistortionEffectSpread(2));
        assert_eq!(
            preset.modulations[7].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::Mix,
            }
        );

        // Third generator
        assert_eq!(
            preset.modulations[8].destination,
            ModulationTarget::Generator {
                generator_id: 3,
                target: GeneratorTarget::Drive,
            }
        );
        assert_eq!(
            preset.modulations[9].destination,
            ModulationTarget::Generator {
                generator_id: 3,
                target: GeneratorTarget::Bias,
            }
        );
        assert_eq!(
            preset.modulations[10].destination,
            DistortionEffectSpread(2)
        );
        assert_eq!(
            preset.modulations[11].destination,
            ModulationTarget::Generator {
                generator_id: 2,
                target: GeneratorTarget::Mix,
            }
        );
    }

    #[test]
    fn note_to_envelope_output() {
        let preset = read_preset("modulation", "note-to-envelope_output-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Attack,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::AttackCurve,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Decay,
            }
        );
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::DecayFalloff,
            }
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Sustain,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Release,
            }
        );
        assert_eq!(
            preset.modulations[6].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::ReleaseFalloff,
            }
        );
        assert_eq!(
            preset.modulations[7].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Delay,
            }
        );
        assert_eq!(
            preset.modulations[8].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Hold,
            }
        );
        assert_eq!(
            preset.modulations[9].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::OutputGain,
            }
        );
        assert_eq!(
            preset.modulations[10].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pan,
            }
        );
    }

    #[test]
    fn note_to_filter_effect() {
        let preset = read_preset("modulation", "note-to-filter_effect-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Cutoff,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Resonance,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Gain,
            }
        );
    }

    // #[test]
    fn _note_to_granular_generators() {
        let preset = read_preset("modulation", "note-to-granular_generators-2.1.0.phaseplant");

        for generator_id in (1 as GeneratorId)..=3 {
            let modulation_offset = ((generator_id - 1) * 13) as usize;
            assert_eq!(
                preset.modulations[modulation_offset + 0].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::Position,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 1].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::GrainLength,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 2].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::AttackCurve,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 3].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::AttackTime,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 4].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::DecayTime,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 5].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::DecayCurve,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 6].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::Grains,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 7].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::RandomPosition,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 8].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::RandomTiming,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 9].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::RandomPitch,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 10].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::RandomLevel,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 11].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::RandomPan,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 12].destination,
                ModulationTarget::Granular {
                    generator_id,
                    target: GranularTarget::RandomReverse,
                }
            );
            assert_eq!(
                preset.modulations[modulation_offset + 13].destination,
                ModulationTarget::Granular {
                    generator_id: 1,
                    target: GranularTarget::Level,
                }
            );
        }
    }

    #[test]
    fn note_to_mix_router() {
        let preset = read_preset("modulation", "note-to-mix_routing-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::RoutingLevel,
            }
        );
    }

    #[test]
    fn note_to_noise_generator() {
        let preset = read_preset("modulation", "note-to-noise_generator-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Slope,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Stereo,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Level,
            }
        );
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pitch,
            }
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Harmonic,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Frequency,
            }
        );
        assert_eq!(
            preset.modulations[6].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::PhaseOffset,
            }
        );
    }

    #[test]
    fn note_to_sample_player() {
        let preset = read_preset("modulation", "note-to-sample_player-2.1.0.phaseplant");

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::StartPos,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Level,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pitch,
            }
        );
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Harmonic,
            }
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Frequency,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::PhaseOffset,
            }
        );
    }

    #[test]
    fn note_to_wavetable_oscillator() {
        let preset = read_preset(
            "modulation",
            "note-to-wavetable_oscillator-2.1.0.phaseplant",
        );

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        assert_eq!(
            preset.modulations[0].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Frame,
            }
        );
        assert_eq!(
            preset.modulations[1].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Bandlimit,
            }
        );
        assert_eq!(
            preset.modulations[2].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Level,
            }
        );
        assert_eq!(
            preset.modulations[3].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Pitch,
            }
        );
        assert_eq!(
            preset.modulations[4].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Harmonic,
            }
        );
        assert_eq!(
            preset.modulations[5].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::Frequency,
            }
        );
        assert_eq!(
            preset.modulations[6].destination,
            ModulationTarget::Generator {
                generator_id: 1,
                target: GeneratorTarget::PhaseOffset,
            }
        );
    }

    // #[test]
    fn _random_generator_levels() {
        // A random modulator going to the levels of five noise generators.
        let preset = read_preset(
            "modulation",
            "random-to-same-generators-levels-2.1.0.phaseplant",
        );
        assert_eq!(preset.modulations.len(), 5);
        for modulation in preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
            assert!(matches!(
                modulation.destination,
                ModulationTarget::Generator {
                    generator_id: _,
                    target: GeneratorTarget::Level
                }
            ));
        }

        // A random modulator going to the levels of five different generators.
        let preset = read_preset(
            "modulation",
            "random-to-different-generators-levels-2.1.0.phaseplant",
        );
        assert_eq!(preset.modulations.len(), 5);
        for modulation in preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
            assert!(matches!(
                modulation.destination,
                ModulationTarget::Generator {
                    generator_id: _,
                    target: GeneratorTarget::Level
                }
            ));
        }
    }

    /// Check that modulator IDs don't overlap
    // #[test]
    fn _ranges() {
        let curve_output_rate_range =
            ModulationTarget::CURVE_OUTPUT_RATE_START..ModulationTarget::CURVE_OUTPUT_RATE_END;
        let generator_range = ModulationTarget::GENERATOR_START..ModulationTarget::GENERATOR_END;
        let granular_range = ModulationTarget::GRANULAR_START..ModulationTarget::GRANULAR_END;
        let lane_range = ModulationTarget::LANE_START..ModulationTarget::LANE_END;
        let macro_range =
            ModulationTarget::MACRO_CONTROL_START..ModulationTarget::MACRO_CONTROL_END;
        let modulator_range = ModulationTarget::MODULATOR_START..ModulationTarget::MODULATOR_END;

        let mut all_ranges = vec![
            curve_output_rate_range,
            generator_range,
            granular_range,
            lane_range,
            macro_range,
            modulator_range,
        ];
        all_ranges.sort_by(|a, b| a.start.cmp(&b.start));
        for (index, range) in all_ranges.iter().enumerate().skip(1) {
            let previous_range = &all_ranges[index - 1];
            assert!(
                range.start > previous_range.end,
                "Range {range:?} conflicts with the previous range {previous_range:?}"
            );
        }
    }

    // #[test]
    fn _scale_to_granular_generators() {
        let preset = read_preset(
            "modulation",
            "scale-to-5-granular_generators-grains-2.1.0.phaseplant",
        );

        for modulation in &preset.modulations {
            assert_eq!(modulation.source, ModulationSource::ModulatorDepth(0));
        }

        for index in 0..5 {
            assert_eq!(
                preset.modulations[0].destination,
                ModulationTarget::Granular {
                    generator_id: index + 1,
                    target: GranularTarget::Grains,
                }
            );
        }
    }

    /// The preset has 32 random modulators where there is a modulation from
    /// each to the global detune.
    #[test]
    fn thirty_two_modulators_to_detune() {
        let preset = read_preset(
            "modulation",
            "modulators-32_random_to_detune-2.1.0.phaseplant",
        );
        assert_eq!(preset.modulations.len(), 32);
        for (id, modulation) in preset.modulations.into_iter().enumerate() {
            assert!(modulation.enabled);
            assert_eq!(
                modulation.source,
                ModulationSource::ModulatorDepth(id as ModulatorId)
            );
            assert_eq!(modulation.destination, ModulationTarget::Detune);
        }
    }
}

//! [Phase Plant](https://kilohearts.com/products/phase_plant) is a virtual
//! synth by Kilohearts. It stores presets in a proprietary binary format.
//!
//! Phase Plant presets can be combined into a bank using the
//! [`kibank`](https://crates.io/crates/kibank) application and library.

use std::fmt::{Display, Formatter};
use std::io::{Error, ErrorKind};

use strum_macros::FromRepr;
use uom::num::Zero;
use uom::si::f32::{Frequency, Ratio, Time};
use uom::si::ratio::percent;

pub use decibels::*;
pub use envelope::*;
pub use io::*;
pub use macro_control::*;
pub use metadata::*;
pub use point::*;
pub use snapin::*;
pub use unison::*;
pub use version::*;

use crate::effect::Effect;
use crate::generator::{Generator, GeneratorId};
use crate::modulation::Modulation;
use crate::modulator::{Modulator, ModulatorContainer};
use crate::version::Version;

mod decibels;
pub mod effect;
mod envelope;
pub mod generator;
mod io;
mod macro_control;
mod metadata;
pub mod modulation;
pub mod modulator;
mod point;
mod snapin;
mod text;
mod unison;
mod version;

pub const LANE_COUNT: usize = 3;

/// Number of generators. Unused generators in the file are ignored.
const GENERATORS_MAX: GeneratorId = 32;

/// Upper limit on the size of the JSON metadata. The length is stored as a u32 so it
/// could be use as a denial of service if there was no other limit.
const METADATA_LENGTH_MAX: usize = 64 * 1024;

/// Number of modulator blocks. Unused modulator blocks in the file are ignored.
const MODULATORS_MAX: usize = 32;

/// Each modulator is allocated 100 bytes plus a plus a header.
const MODULATOR_BLOCK_SIZE: usize = 100;

/// How many parts are allowed when specifying a path.
const PATH_COMPONENT_COUNT_MAX: usize = 100; // TODO: Operating system limit?

/// Length of a note.
///
/// See also: [`PatternResolution`](effect::PatternResolution)
#[derive(Clone, Copy, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum NoteValue {
    // The discriminants correspond to the file format.
    Quarter,
    QuarterTriplet,
    Eighth,
    EightTriplet,
    Sixteenth,
    SixteenthTriplet,
    ThirtySecond,
    ThirtySecondTriplet,
    SixtyFourth,
}

impl NoteValue {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown note value {id}")))
    }
}

impl Display for NoteValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use NoteValue::*;
        let msg = match self {
            Quarter => "1/4",
            QuarterTriplet => "1/4T",
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

/// A rate defines the speed of an operation. It can be determined by frequency
/// or based on the song tempo.
#[derive(Clone, Debug, PartialEq)]
pub struct Rate {
    pub frequency: Frequency,
    pub numerator: u32,
    pub denominator: NoteValue,

    /// If not set the rate time is used otherwise the time signature is used,
    /// made up of the rate numerator and denominator is used.
    pub sync: bool,
}

#[derive(Copy, Clone, Debug, Default, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum LaneDestination {
    // The discriminants correspond to the file format. They are not the same
    // as the destination used by `OutputGenerator`.
    Lane2 = 2,
    Lane3 = 0,
    #[default]
    Master = 1,
    Lane1 = 3,
    // FIXME: Value is guessed, it's not in the data for the lanes, will find it in the noise generator
    Sideband = 5,
}

impl LaneDestination {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown lane destination {id}"),
            )
        })
    }
}

impl Display for LaneDestination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use LaneDestination::*;
        let msg = match self {
            Lane2 => "Lane 2",
            Lane3 => "Lane 3",
            Master => "Master",
            Lane1 => "Lane 1",
            Sideband => "Sideband",
        };
        f.write_str(msg)
    }
}

#[derive(Debug, PartialEq)]
pub struct Lane {
    pub enabled: bool,

    /// There is no restriction on the number of snapins.
    pub snapins: Vec<Snapin>,

    pub destination: LaneDestination,

    /// How many lanes from left to right are poly.
    pub poly_count: u8,

    pub mute: bool,
    pub solo: bool,
    pub gain: f32,
    pub mix: Ratio,
}

impl Lane {
    pub const COUNT: u8 = 3;

    /// Find the first snapin that has an effect with the given type.
    pub fn find_effect<T: Effect>(&self) -> Option<(&Snapin, &T)> {
        // Returns the effect so it's already the right type
        self.snapins
            .iter()
            .find(|snapin| snapin.effect.downcast_ref::<T>().is_some())
            .map(|snapin| (snapin, snapin.effect.downcast_ref::<T>().unwrap()))
    }
}

impl Default for Lane {
    fn default() -> Self {
        Lane {
            enabled: true,
            snapins: Vec::new(),
            destination: LaneDestination::Master,
            poly_count: 0,
            mute: false,
            solo: false,
            gain: 1.0,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Preset {
    pub format_version: Version<u32>,
    pub generators: Vec<Box<dyn Generator>>,

    pub mod_wheel_value: Ratio,

    #[doc(alias = "portamento")]
    pub glide_enabled: bool,
    pub glide_time: f32,

    #[doc(alias = "glide_auto")]
    pub glide_legato: bool,

    pub lanes: Vec<Lane>,

    // TODO: Switch to an array because the number of controls is known.
    pub macro_controls: Vec<MacroControl>,

    /// Linear
    pub master_gain: f32,

    pub master_pitch: f32,
    pub metadata: Metadata,
    pub modulations: Vec<Modulation>,
    pub modulator_containers: Vec<ModulatorContainer>,
    pub polyphony: u32,

    /// When enabled LFO restarts for each new voice, disabled all voices share a global LFO.
    pub retrigger_enabled: bool,
    pub unison: Unison,
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            format_version: WRITE_SAME_AS.format_version(),
            mod_wheel_value: Ratio::zero(),
            glide_enabled: false,
            glide_legato: false,
            glide_time: 0.0,
            generators: Vec::new(),
            lanes: vec![
                Lane {
                    destination: LaneDestination::Lane2,
                    ..Default::default()
                },
                Lane {
                    destination: LaneDestination::Lane3,
                    ..Default::default()
                },
                Lane {
                    destination: LaneDestination::Master,
                    ..Default::default()
                },
            ],
            macro_controls: (1..=MacroControl::COUNT)
                .map(|n| MacroControl::new(format!("Macro {}", n)))
                .collect(),
            master_gain: Decibels::ZERO.linear(),
            master_pitch: 0.0,
            metadata: Default::default(),
            modulations: Vec::new(),
            modulator_containers: Vec::new(),
            polyphony: 8,
            retrigger_enabled: true,
            unison: Default::default(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::fs::File;
    use std::io;
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};
    use std::path::Path;

    use crate::tests::test_data_path;
    use crate::*;

    fn load_preset(components: &[&str]) -> io::Result<Preset> {
        let mut path = test_data_path(&[]);
        if !path.exists() {
            panic!("Phase Plant test data path does not exist: {path:?}");
        }

        for component in components {
            path = path.join(component);
        }
        Preset::read_file(&path)
    }

    /// If set a file will be created if reading the preset back fails. Useful
    /// for examining the preset that could not be reloaded.
    #[allow(dead_code)]
    const RELOAD_CREATES_FILE_ON_READ_ERROR: bool = true;

    /// If set a file will be created if writing a preset fails. Useful for
    /// examining the preset that could not be reloaded.
    #[allow(dead_code)]
    const RELOAD_CREATES_FILE_ON_WRITE_ERROR: bool = true;

    /// Return a version of the preset that has been gone through the writing
    /// process. Some basic assertions will be made comparing the before and
    /// after presets. Individual tests must still check for the correct
    /// outcome.
    #[must_use]
    pub(crate) fn read_preset(dir_name: &str, file_name: &str) -> Preset {
        load_preset(&[dir_name, file_name]).expect("preset")
        // TODO: Disabled until write completed.
        // return rewrite_preset(&preset, file_name)
    }

    pub(crate) fn read_effect_preset(effect_name: &str, file_name: &str) -> io::Result<Preset> {
        let preset = load_preset(&["effects", effect_name, file_name])?;
        // TODO: Disabled until write completed.
        // return rewrite_preset(&preset, file_name)
        Ok(preset)
    }

    pub(crate) fn read_generator_preset(
        generator_name: &str,
        file_name: &str,
    ) -> io::Result<Preset> {
        let preset = load_preset(&["generators", generator_name, file_name])?;
        // TODO: Disabled until write completed.
        // return rewrite_preset(&preset, file_name)
        Ok(preset)
    }

    pub(crate) fn read_modulator_preset(
        modulator_name: &str,
        file_name: &str,
    ) -> io::Result<Preset> {
        let preset = load_preset(&["modulators", modulator_name, file_name])?;
        // TODO: Disabled until write completed.
        // return rewrite_preset(&preset, file_name)
        Ok(preset)
    }

    fn _rewrite_preset(preset: &Preset, file_name: &str) -> Preset {
        let mut write_cursor = Cursor::new(Vec::with_capacity(16 * 1024));
        match preset.write(&mut write_cursor) {
            Ok(_) => {
                let name_str = Path::new(file_name)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string());

                // Temporarily write out the preset.
                #[cfg(disabled)]
                {
                    write_cursor.seek(SeekFrom::Start(0)).unwrap();
                    let filename = format!(
                        "test-{}-{}.phaseplant",
                        name_str.clone().unwrap_or_default(),
                        uuid::Uuid::new_v4()
                    );
                    let path = std::env::temp_dir().join(&filename);
                    let mut file = File::create(&path).expect("Create file");
                    let mut out = Vec::with_capacity(write_cursor.position() as usize);
                    write_cursor.seek(SeekFrom::Start(0)).unwrap();
                    write_cursor.read_to_end(&mut out).unwrap();
                    file.write_all(&out).unwrap();
                    println!("Test preset written to {}", path.to_string_lossy());
                }

                write_cursor.seek(SeekFrom::Start(0)).unwrap();

                #[cfg(disabled)]
                {
                    let mut file = File::create("/tmp/reload.phaseplant").expect("Create file");
                    let mut out = Vec::with_capacity(write_cursor.position() as usize);
                    write_cursor.seek(SeekFrom::Start(0)).unwrap();
                    write_cursor.read_to_end(&mut out).unwrap();
                    file.write_all(&out).unwrap();
                    panic!("Debug file written to {file:?}");
                }

                match Preset::read(&mut write_cursor, name_str) {
                    Ok(written) => {
                        // The entire presets can't be compared because of floating point equality.

                        // The name entire metadata cannot be compared because Phase Plant doesn't
                        // persist the preset name, it uses the filename.
                        assert_eq!(preset.metadata.description, written.metadata.description);
                        assert_eq!(preset.metadata.author, written.metadata.author);
                        assert_eq!(preset.metadata.category, written.metadata.category);

                        assert_eq!(preset.macro_controls, written.macro_controls);

                        assert_eq!(preset.polyphony, written.polyphony);
                        assert_eq!(preset.unison, written.unison);

                        assert_eq!(
                            preset.generators.len(),
                            written.generators.len(),
                            "number of generators"
                        );
                        assert_eq!(preset.lanes.len(), written.lanes.len(), "number of lanes");
                        assert_eq!(
                            preset.macro_controls.len(),
                            written.macro_controls.len(),
                            "number of macro controls"
                        );
                        assert_eq!(
                            preset.modulator_containers.len(),
                            written.modulator_containers.len(),
                            "number of modulators"
                        );

                        written
                    }
                    Err(error) => {
                        if RELOAD_CREATES_FILE_ON_READ_ERROR {
                            let filename =
                                format!("reload-read-error-{}.phaseplant", uuid::Uuid::new_v4());
                            let path = std::env::temp_dir().join(filename);
                            let mut file = File::create(&path).expect("Create file");

                            let mut out = Vec::with_capacity(write_cursor.position() as usize);
                            write_cursor.seek(SeekFrom::Start(0)).unwrap();
                            write_cursor.read_to_end(&mut out).unwrap();
                            file.write_all(&out).unwrap();
                            panic!(
                                "{:?} - debug file written to {}",
                                error,
                                path.to_string_lossy()
                            );
                        }
                        panic!("{:?}", error);
                    }
                }
            }
            Err(error) => {
                if RELOAD_CREATES_FILE_ON_WRITE_ERROR {
                    let filename =
                        format!("reload-write-error-{}.phaseplant", uuid::Uuid::new_v4());
                    let path = std::env::temp_dir().join(filename);
                    let mut file = File::create(&path).expect("Create file");

                    let mut out = Vec::with_capacity(write_cursor.position() as usize);
                    write_cursor.seek(SeekFrom::Start(0)).unwrap();
                    write_cursor.read_to_end(&mut out).unwrap();
                    file.write_all(&out).unwrap();
                    panic!(
                        "{:?} - debug file written to {}",
                        error,
                        path.to_string_lossy()
                    );
                }
                panic!("{:?}", error);
            }
        }
    }

    #[test]
    fn default() {
        let preset = Preset::default();
        assert_eq!(preset.format_version.major, 6);

        // FIXME: USE FOR INIT PRESET
        let metadata = &preset.metadata;
        assert!(metadata.author.is_none());
        assert!(metadata.description.is_none());
        assert!(metadata.name.is_none());

        assert!(!preset.glide_enabled);
        assert!(!preset.glide_legato);
        assert_eq!(preset.glide_time, 0.0);

        assert_eq!(preset.lanes.len(), 3);
        preset.lanes.iter().for_each(|lane| {
            assert!(lane.enabled);
            assert_eq!(lane.poly_count, 0);
            assert!(!lane.mute);
            assert_eq!(lane.gain, 1.0);
            assert_eq!(lane.mix.get::<percent>(), 100.0);
        });
        assert_eq!(preset.lanes[0].destination, LaneDestination::Lane2);
        assert_eq!(preset.lanes[1].destination, LaneDestination::Lane3);
        assert_eq!(preset.lanes[2].destination, LaneDestination::Master);

        assert_eq!(preset.macro_controls.len(), 8);
        assert_eq!(preset.macro_controls[0].name, "Macro 1");
        assert_eq!(preset.macro_controls[1].name, "Macro 2");
        assert_eq!(preset.macro_controls[2].name, "Macro 3");
        assert_eq!(preset.macro_controls[3].name, "Macro 4");
        assert_eq!(preset.macro_controls[4].name, "Macro 5");
        assert_eq!(preset.macro_controls[5].name, "Macro 6");
        assert_eq!(preset.macro_controls[6].name, "Macro 7");
        assert_eq!(preset.macro_controls[7].name, "Macro 8");

        assert_eq!(preset.master_gain, Decibels::ZERO.linear());
        assert_eq!(preset.metadata.author, None);
        assert_eq!(preset.metadata.category, None);
        assert_eq!(preset.metadata.description, None);
        assert_eq!(preset.metadata.name, None);
        assert_eq!(preset.polyphony, 8);
        assert!(preset.retrigger_enabled);

        assert!(preset.lanes[0].snapins.is_empty());

        let unison = &preset.unison;
        assert!(!unison.enabled);
        assert_eq!(unison.voices, 4);
        assert_eq!(unison.mode, UnisonMode::Smooth);
        assert_eq!(unison.detune, 25.0);
        assert_eq!(unison.spread, 0.0);
        assert_eq!(unison.blend, 1.0);
        assert_eq!(unison.bias, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    pub(crate) fn test_data_path(components: &[&str]) -> PathBuf {
        let mut parts = vec!["tests"];
        parts.extend_from_slice(components);
        parts.iter().collect::<PathBuf>()
    }
}

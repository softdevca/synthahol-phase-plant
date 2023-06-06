//! Snapins are containers for effects.

use crate::effect::{Effect, Filter};
use crate::io::WRITE_SAME_AS;
use crate::Metadata;
use crate::version::Version;

type SnapinPosition = u16;

#[derive(Debug)]
pub struct Snapin {
    pub name: String,
    pub metadata: Metadata,
    pub enabled: bool,
    pub minimized: bool,

    /// Where in the lane the snapin lives. The minimum position is [`Snapin::MIN_POSITION`].
    pub position: SnapinPosition,

    /// Phase Plant version 1.7 does not use a preset path and stores the
    /// path as the name.
    pub preset_name: String,

    pub preset_path: Vec<String>,

    /// If the effect has a preset selected but it has been edited.
    pub preset_edited: bool,

    pub host_version: Version<u8>,
    pub effect_version: u32,
    pub effect: Box<dyn Effect>,
}

impl Snapin {
    pub const MIN_POSITION: SnapinPosition = 1;

    /// Create a snapin that contains the effect.
    pub fn new(
        effect: Box<dyn Effect>,
        position: SnapinPosition,
        enabled: bool,
        minimized: bool,
    ) -> Snapin {
        Snapin {
            name: effect.mode().name().to_owned(),
            effect_version: effect.write_version(),
            effect,
            enabled,
            minimized,
            position,
            ..Default::default()
        }
    }

    /// Update the position field of the snapin to match the order they are in the list.
    pub fn position_by_index(snapins: &mut [Snapin]) {
        for (position, snapin) in snapins.iter_mut().enumerate() {
            snapin.position = position as SnapinPosition + Snapin::MIN_POSITION;
        }
    }
}

/// If you are creating a `Snapin` without using `Snapin::from` then you will
/// need to set the effect version depending on the effect the Snapin will
/// contain.
impl Default for Snapin {
    fn default() -> Self {
        Self {
            name: Default::default(),
            enabled: true,
            minimized: false,
            position: 0,
            metadata: Metadata::default(),
            preset_name: String::default(),
            preset_path: Vec::new(),
            preset_edited: false,
            host_version: WRITE_SAME_AS.version(),
            effect_version: 0,
            effect: Box::<Filter>::default(),
        }
    }
}

impl PartialEq for Snapin {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled
            && self.minimized == other.minimized
            && self.name == other.name
            && self.host_version == other.host_version
            && self.effect_version == other.effect_version
            && self.preset_name == other.preset_name
            && self.preset_path == other.preset_path
            && self.effect.box_eq(&other.effect)
    }
}

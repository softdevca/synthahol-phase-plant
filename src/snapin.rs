//! Snapins are containers for effects.

use crate::effect::{Effect, EffectVersion, Filter};
use crate::io::WRITE_SAME_AS;
use crate::version::Version;
use crate::Metadata;

pub type SnapinId = u16;

#[derive(Debug)]
pub struct Snapin {
    /// Unique ID of the snapin in the lane. It does not represent the order
    /// of snapins in the lane.
    pub id: SnapinId,

    pub name: String,
    pub metadata: Metadata,
    pub enabled: bool,
    pub minimized: bool,

    /// Position of the group that contains this snapin.
    pub group_id: Option<SnapinId>,

    /// Phase Plant version 1.7 does not use a preset path and stores the
    /// path as the name.
    pub preset_name: String,

    pub preset_path: Vec<String>,

    /// If the effect has a preset selected but it has been edited.
    pub preset_edited: bool,

    pub host_version: Version<u8>,
    pub effect_version: EffectVersion,
    pub effect: Box<dyn Effect>,
}

impl Snapin {
    pub const MIN_POSITION: SnapinId = 1;

    /// Create a snapin that contains the effect.
    pub fn new(effect: Box<dyn Effect>, id: SnapinId, enabled: bool, minimized: bool) -> Snapin {
        Snapin {
            id,
            name: effect.mode().name().to_owned(),
            effect_version: effect.mode().default_version(),
            effect,
            enabled,
            minimized,
            ..Default::default()
        }
    }

    /// Update the identifiers of the snapin to match the order they are in the
    /// list of snapins.
    pub fn update_ids_to_match_order(snapins: &mut [Snapin]) {
        for (id, snapin) in snapins.iter_mut().enumerate() {
            snapin.id = id as SnapinId + Snapin::MIN_POSITION;
        }
    }
}

/// If you are creating a `Snapin` without using `Snapin::from` then you will
/// need to set the effect version depending on the effect the Snapin will
/// contain.
impl Default for Snapin {
    fn default() -> Self {
        Self {
            id: 0,
            name: Default::default(),
            enabled: true,
            minimized: false,
            group_id: None,
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
        self.id == other.id
            && self.enabled == other.enabled
            && self.minimized == other.minimized
            && self.name == other.name
            && self.group_id == other.group_id
            && self.metadata == other.metadata
            && self.host_version == other.host_version
            && self.effect_version == other.effect_version
            && self.preset_name == other.preset_name
            && self.preset_path == other.preset_path
            && self.effect.box_eq(&other.effect)
    }
}

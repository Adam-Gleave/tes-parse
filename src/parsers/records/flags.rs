use std::convert::{TryFrom, TryInto, Infallible};

use bitflags::{bitflags, BitFlags};

pub trait Flags: BitFlags<u32> + TryFrom<u32> + TryInto<u32> + Default {
    fn test(&self, value: u32) -> bool {
        let flags = value.try_into().or(Err(crate::Error::Unexpected)).unwrap();
        self.contains(flags)
    }
}

bitflags! {
    pub struct RecordFlags: u32 {
        const DELETED                   = 0x00000020;
        const CONSTANT                  = 0x00000040;
        const MUST_UPDATE_ANIMS         = 0x00000100;
        const HIDDEN_FROM_LOCAL_MAP     = 0x00000200;
        const QUEST_ITEM                = 0x00000400;
        const INITIALLY_DISABLED        = 0x00000800;
        const IGNORED                   = 0x00001000;
        const VISIBLE_WHEN_DISTANT      = 0x00008000;
        const RANDOM_ANIMATION_START    = 0x00010000;
        const DANGEROUS                 = 0x00020000;
        const COMPRESSED                = 0x00040000;
        const CANNOT_WAIT               = 0x00080000;
        const IGNORE_OBJECT_INTERACTION = 0x00100000;
        const MARKER                    = 0x00800000;
        const OBSTACLE                  = 0x02000000;
        const NAVMESH_GEN_FILTER        = 0x04000000;
        const NAVMESH_GEN_BBOX          = 0x08000000;
        const REFLECTED_BY_WATER        = 0x10000000;
        const NO_HAVOK_SETTLE           = 0x20000000;
        const NO_RESPAWN                = 0x40000000;
        const MULTI_BOUND               = 0x80000000;
    }
}

impl Default for RecordFlags {
    fn default() -> Self {
        RecordFlags::empty()
    }
}

impl TryFrom<u32> for RecordFlags {
    type Error = crate::Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        RecordFlags::from_bits(value).ok_or(crate::Error::InvalidFlags(value))
    }
}

impl TryInto<u32> for RecordFlags {
    type Error = Infallible;

    fn try_into(self) -> Result<u32, Self::Error> {
        Ok(self.bits)
    }
}

impl Flags for RecordFlags {}

bitflags! {
    pub struct PluginFlags: u32 {
        const MASTER    = 0x0001;
        const LOCALIZED = 0x0080;
        const LIGHT     = 0x0200;
    }
}

impl Default for PluginFlags {
    fn default() -> Self {
        PluginFlags::empty()
    }
}

impl TryFrom<u32> for PluginFlags {
    type Error = crate::Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        PluginFlags::from_bits(value).ok_or(crate::Error::InvalidFlags(value))
    }
}

impl TryInto<u32> for PluginFlags {
    type Error = Infallible;

    fn try_into(self) -> Result<u32, Self::Error> {
        Ok(self.bits)
    }
}

impl Flags for PluginFlags {}

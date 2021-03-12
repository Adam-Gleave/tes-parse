use std::convert::TryFrom;

use bitflags::bitflags;

bitflags! {
    pub struct RecordFlags: u32 {
        const MASTER    = 0x0001;
        const LOCALIZED = 0x0080;
        const LIGHT     = 0x0200;
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

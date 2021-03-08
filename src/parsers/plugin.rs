use crate::error;
use super::group;
use super::prelude::*;
use super::records::{record, Record};
use bitflags::bitflags;
use nom::IResult;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Plugin {
    pub tes4: Record<PluginFlags>,
    pub groups: Vec<group::Group>,
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
    type Error = error::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        PluginFlags::from_bits(value).ok_or(error::Error::new(error::ErrorKind::InvalidFlags))
    }
}

pub fn plugin(bytes: &[u8]) -> IResult<&[u8], Plugin> {
    let (mut bytes, tes4) = record::<PluginFlags>(bytes)?;
    let mut groups = vec![];

    while bytes.len() > 0 {
        let (remaining, group) = group::group(bytes)?;
        let (remaining, _) = take(group.size as usize - group::Group::HEADER_SIZE)(remaining)?;

        groups.push(group);
        bytes = remaining;
    }

    Ok((bytes, Plugin { tes4, groups }))
}

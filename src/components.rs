use crate::error;
use bitflags::bitflags;
use std::{convert::TryFrom, fmt::{self, Debug}, ops::{self, Deref}};

pub struct TypeCode([u8; 4]);

impl fmt::Debug for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code_str = std::str::from_utf8(self.deref()).or(Err(fmt::Error))?;
        write!(f, "{}", code_str)
    }
}

impl fmt::Display for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code_str = std::str::from_utf8(self.deref()).or(Err(fmt::Error))?;
        write!(f, "{}", code_str)
    }
}

impl From<[u8; 4]> for TypeCode {
    fn from(code: [u8; 4]) -> Self {
        Self(code)
    }
}

impl From<u32> for TypeCode {
    fn from(code: u32) -> Self {
        Self(code.to_le_bytes())
    }
}

impl ops::Deref for TypeCode {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0 
    }
}

pub struct FormId(u32);

impl From<u32> for FormId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl ops::Deref for FormId {
    type Target = u32;

    fn deref (&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for FormId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08x}", self.deref())
    }
}

impl fmt::Display for FormId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08x}", self.deref())
    }
}

#[derive(Debug)]
pub struct Plugin {
    pub tes4: Record<PluginFlags>,
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

#[derive(Debug)]
pub struct Record<Flags>
where 
    Flags: Debug
{
    pub header: RecordHeader<Flags>,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct RecordHeader<Flags> 
where 
    Flags: Debug 
{
    pub code: TypeCode,
    pub size: u32,
    pub flags: Flags,
    pub id: FormId,
    pub timestamp: u16,
    pub vc_info: u16,
    pub version: u16,
    pub unknown: u16,
}
pub mod file_header;

use crate::error::{Error, ErrorKind};
use crate::parsers::common::*;
use crate::parsers::prelude::*;
use bitflags::bitflags;
use std::convert::TryFrom;
use std::fmt::Debug;

pub type FileHeaderRecord = GenericRecord<PluginFlags>;
pub type Record = GenericRecord<RecordFlags>;

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
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        RecordFlags::from_bits(value).ok_or(Error::new(ErrorKind::InvalidFlags))
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
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        PluginFlags::from_bits(value).ok_or(Error::new(ErrorKind::InvalidFlags))
    }
}

#[derive(Debug)]
pub struct GenericRecord<Flags>
where 
    Flags: Debug
{
    pub header: RecordHeader<Flags>,
    pub data: RecordData,
}

pub(crate) fn record(bytes: &[u8]) -> IResult<&[u8], Record> {
    let (bytes, header) = header::<RecordFlags>(bytes)?;
    let (bytes, data) = data::<RecordFlags>(bytes, &header)?;
    
    Ok((bytes, Record { header, data }))
}

pub(crate) fn file_header_record(bytes: &[u8]) -> IResult<&[u8], FileHeaderRecord> {
    let (bytes, header) = header::<PluginFlags>(bytes)?;
    let (bytes, data) = data::<PluginFlags>(bytes, &header)?;

    Ok((bytes, FileHeaderRecord { header, data }))
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

fn header<Flags>(bytes: &[u8]) -> IResult<&[u8], RecordHeader<Flags>>
where
    Flags: TryFrom<u32> + Debug + Default
{
    map(
        tuple((le_u32, le_u32, le_u32, le_u32, le_u16, le_u16, le_u16, le_u16)),
        |(code, size, flags, id, timestamp, vc_info, version, unknown)| RecordHeader::<Flags> {
            code: code.into(),
            size,
            flags: Flags::try_from(flags).unwrap_or(Flags::default()),
            id: id.into(),
            timestamp,
            vc_info,
            version,
            unknown,
        }
    )(bytes)
}

#[derive(Debug)]
pub enum RecordData {
    FileHeader(file_header::FileHeaderData),
    Unknown(Vec<u8>),
}

fn data<'a, Flags>(bytes: &'a [u8], header: &RecordHeader<Flags>) -> IResult<&'a [u8], RecordData>
where
    Flags: Debug 
{
    let (bytes, data_bytes) = take(header.size)(bytes)?;

    match header.code.to_string().as_ref() {
        "TES4" => Ok((bytes, map(file_header::data, |data| RecordData::FileHeader(data))(data_bytes)?.1)),
        _ => Ok((bytes, RecordData::Unknown(data_bytes.to_vec()))),
    }
}
pub mod file_header;
pub mod flags;

use std::{convert::TryFrom, fmt::Debug};

use crate::parsers::common::{FormId, subrecords, TypeCode};

use log::debug;
use nom::{
    bytes::complete::take,
    combinator::map,
    number::complete::{le_u16, le_u32},
    sequence::tuple,
};

pub type FileHeaderRecord = GenericRecord<flags::PluginFlags>;
pub type Record = GenericRecord<flags::RecordFlags>;

#[derive(Debug)]
pub struct GenericRecord<Flags>
where
    Flags: Debug,
{
    pub header: RecordHeader<Flags>,
    pub data: RecordData,
}

pub(crate) fn record(bytes: &[u8]) -> crate::IResult<&[u8], (String, Record)> {
    let (bytes, header) = header::<flags::RecordFlags>(bytes)?;
    let (bytes, data) = data::<flags::RecordFlags>(bytes, &header)?;

    debug!("EditorID: {}", data.0);

    Ok((
        bytes,
        (
            data.0,
            Record {
                header,
                data: data.1,
            },
        ),
    ))
}

pub(crate) fn file_header_record(bytes: &[u8]) -> crate::IResult<&[u8], FileHeaderRecord> {
    let (bytes, header) = header::<flags::PluginFlags>(bytes)?;
    let (bytes, data) = data::<flags::PluginFlags>(bytes, &header)?;

    Ok((
        bytes,
        FileHeaderRecord {
            header,
            data: data.1,
        },
    ))
}

#[derive(Debug)]
pub struct RecordHeader<Flags>
where
    Flags: Debug,
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

fn header<Flags>(bytes: &[u8]) -> crate::IResult<&[u8], RecordHeader<Flags>>
where
    Flags: TryFrom<u32> + Debug + Default,
{
    map(
        tuple((
            le_u32, le_u32, le_u32, le_u32, le_u16, le_u16, le_u16, le_u16,
        )),
        |(code, size, flags, id, timestamp, vc_info, version, unknown)| RecordHeader::<Flags> {
            code: code.into(),
            size,
            flags: Flags::try_from(flags).unwrap_or(Flags::default()),
            id: id.into(),
            timestamp,
            vc_info,
            version,
            unknown,
        },
    )(bytes)
}

#[derive(Debug)]
pub enum RecordData {
    FileHeader(file_header::FileHeaderData),
    Unknown(Vec<u8>),
}

fn data<'a, Flags>(
    bytes: &'a [u8],
    header: &RecordHeader<Flags>,
) -> crate::IResult<&'a [u8], (String, RecordData)>
where
    Flags: Debug,
{
    let (bytes, data_bytes) = take(header.size)(bytes)?;

    match header.code.to_string().as_ref() {
        "TES4" => Ok((
            bytes,
            map(file_header::data, |data| {
                (String::new(), RecordData::FileHeader(data))
            })(data_bytes)?
            .1,
        )),
        _ => Ok((
            bytes,
            map(unknown_data, |(edid, data)| {
                (edid, RecordData::Unknown(data))
            })(data_bytes)?
            .1,
        )),
    }
}

fn unknown_data(bytes: &[u8]) -> crate::IResult<&[u8], (String, Vec<u8>)> {
    let record_data = bytes.clone().to_vec();
    let (remaining, subrecords) = subrecords(bytes)?;

    if let Some(first_subrecord) = subrecords.first() {
        if first_subrecord.code.to_string().as_str() == "EDID" {
            Ok((
                remaining,
                (
                    String::from_utf8(first_subrecord.code.to_vec()).unwrap(),
                    record_data,
                ),
            ))
        } else {
            Ok((remaining, (String::from("Missing EditorID"), record_data)))
        }
    } else {
        Ok((remaining, (String::from("Compressed Record"), record_data)))
    }
}

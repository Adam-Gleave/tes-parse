pub mod file_header;
pub mod flags;

use std::{fmt::Debug, io::Read};

use crate::parsers::common::{subrecords, FormId, TypeCode};
use flags::{Flags, RecordFlags};

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
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

pub(crate) fn record(bytes: &[u8]) -> crate::IResult<&[u8], Record> {
    let (bytes, mut header) = header::<flags::RecordFlags>(bytes)?;
    let (bytes, (editor_id, data)) = data::<flags::RecordFlags>(bytes, &header)?;

    log::debug!("Loaded editor_id: {}", editor_id);

    header.editor_id = Some(editor_id);

    Ok((bytes, (Record { header, data })))
}

pub(crate) fn file_header_record(bytes: &[u8]) -> crate::IResult<&[u8], FileHeaderRecord> {
    let (bytes, header) = header::<flags::PluginFlags>(bytes)?;
    let (bytes, data) = data::<flags::PluginFlags>(bytes, &header)?;

    Ok((bytes, FileHeaderRecord { header, data: data.1 }))
}

#[derive(Debug)]
pub struct RecordHeader<F>
where
    F: Debug,
{
    pub code: TypeCode,
    pub size: u32,
    pub flags: F,
    pub id: FormId,
    pub timestamp: u16,
    pub vc_info: u16,
    pub version: u16,
    pub unknown: u16,
    pub editor_id: Option<String>,
}

fn header<F>(bytes: &[u8]) -> crate::IResult<&[u8], RecordHeader<F>>
where
    F: Flags,
{
    map(
        tuple((le_u32, le_u32, le_u32, le_u32, le_u16, le_u16, le_u16, le_u16)),
        |(code, size, flags, id, timestamp, vc_info, version, unknown)| RecordHeader::<F> {
            code: code.into(),
            size,
            flags: F::try_from(flags).unwrap_or(F::default()),
            id: id.into(),
            timestamp,
            vc_info,
            version,
            unknown,
            editor_id: None,
        },
    )(bytes)
}

#[derive(Debug)]
pub enum RecordData {
    FileHeader(file_header::FileHeaderData),
    Unknown(Vec<u8>),
}

fn data<'a, F>(bytes: &'a [u8], header: &RecordHeader<F>) -> crate::IResult<&'a [u8], (String, RecordData)>
where
    F: Flags,
{
    let (bytes, data_bytes) = take(header.size)(bytes)?;

    match header.code.to_string().as_ref() {
        "TES4" => {
            let (_, data) = file_header::data(data_bytes)?;
            Ok((bytes, (String::new(), RecordData::FileHeader(data))))
        }
        _ => {
            let (_, (editor_id, data)) = unknown_data(data_bytes, header)?;
            Ok((bytes, (editor_id, RecordData::Unknown(data))))
        }
    }
}

fn unknown_data<'a, F>(bytes: &'a [u8], header: &RecordHeader<F>) -> crate::IResult<&'a [u8], (String, Vec<u8>)>
where
    F: Flags,
{
    let mut record_data = bytes.to_vec();

    if header.flags.test(RecordFlags::COMPRESSED.bits()) {
        record_data = decompress(record_data, header.size).unwrap();
    }

    let (_, subrecords) = subrecords(record_data.as_slice())?;

    if let Some(first_subrecord) = subrecords.first() {
        if first_subrecord.code.to_string().as_str() == "EDID" {
            let editor_id = String::from_utf8(first_subrecord.data.to_vec()).unwrap();
            Ok((&[], (editor_id, record_data)))
        } else {
            Ok((&[], (String::from("Missing EditorID"), record_data)))
        }
    } else {
        Ok((&[], (String::from("Compressed Record"), record_data)))
    }
}

fn decompress(mut bytes: Vec<u8>, size: u32) -> Result<Vec<u8>, crate::Error> {
    let decompressed_size = bytes.drain(0..4).as_slice().read_u32::<LittleEndian>().unwrap();
    let decoder = ZlibDecoder::new(bytes.as_slice());
    let mut decompressed = vec![];

    log::debug!("Decompressing record, expecting {} bytes", decompressed_size);
    decoder.take(size as u64).read_to_end(&mut decompressed).unwrap();

    Ok(decompressed)
}

use crate::parsers::common::*;
use crate::parsers::prelude::*;

#[derive(Debug, Default)]
pub struct FileHeaderData {
    pub hedr: Hedr,
    pub author: Option<String>,
    pub description: Option<String>,
    pub masters: Vec<MasterFile>,
    pub overrides: Vec<FormId>,
    pub intv: u32,
    pub incc: u32,
}

pub(super) fn data(bytes: &[u8]) -> IResult<&[u8], FileHeaderData> {
    let mut record_data = FileHeaderData::default();
    let (bytes, subrecords) = subrecords(bytes)?;

    for subrecord in subrecords {
        let code = subrecord.0.to_string();
        let bytes = subrecord.1;

        match code.as_str() {
            "HEDR" => { record_data.hedr        = hedr(bytes)?.1; }
            "CNAM" => { record_data.author      = Some(zstring(bytes)?.1); },
            "SNAM" => { record_data.description = Some(zstring(bytes)?.1); },
            "MAST" => { record_data.masters     = many0(mast)(bytes)?.1; }
            "ONAM" => { record_data.overrides   = many0(form_id)(bytes)?.1; },
            "INTV" => { record_data.intv        = le_u32(bytes)?.1; },
            "INCC" => { record_data.incc        = le_u32(bytes)?.1; },
            _ => (),
        }
    } 

    Ok((bytes, record_data))
}

#[derive(Debug, Default)]
pub struct Hedr {
    pub version: f32,
    pub num_records: i32,
    pub next_id: FormId,
}

fn hedr(bytes: &[u8]) -> IResult<&[u8], Hedr> {
    map(
        tuple((le_f32, le_i32, le_u32)), 
        |(version, num_records, next_id)| Hedr { 
            version, 
            num_records, 
            next_id: next_id.into(), 
        }
    )(bytes)
}

#[derive(Debug, Default)]
pub struct MasterFile {
    pub name: String,
    pub tag: u64,
}

fn mast(bytes: &[u8]) -> IResult<&[u8], MasterFile> {
    map(
        tuple((zstring, le_u64)),
        |(name, tag)| MasterFile { name, tag }
    )(bytes)
}
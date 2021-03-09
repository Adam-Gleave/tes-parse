use super::common::TypeCode;
use super::group;
use super::records;
use nom::IResult;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Plugin {
    pub tes4: records::FileHeaderRecord,
    pub groups: HashMap<TypeCode, group::Group>,
}

pub fn plugin(bytes: &[u8]) -> IResult<&[u8], Plugin> {
    let (mut bytes, tes4) = records::file_header_record(bytes)?;
    let mut groups = HashMap::new();

    while bytes.len() > 0 {
        let (remaining, (code, group)) = group::top_group(bytes)?;
        groups.insert(code, group);
        bytes = remaining;
    }

    Ok((bytes, Plugin { tes4, groups }))
}

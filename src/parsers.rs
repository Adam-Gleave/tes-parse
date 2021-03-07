use crate::components::*;
use nom::IResult;
use nom::combinator::map;
use nom::bytes::complete::take;
use nom::number::complete::{le_u16, le_u32};
use nom::sequence::tuple;
use std::convert::TryFrom;
use std::fmt::Debug;

pub fn plugin(bytes: &[u8]) -> IResult<&[u8], Plugin> {
    record::<PluginFlags>(bytes).map(|(bytes, tes4)| (bytes, Plugin { tes4 }))
}

pub fn record<Flags>(bytes: &[u8]) -> IResult<&[u8], Record<Flags>>
where
    Flags: TryFrom<u32> + Debug + Default
{
    let (bytes, header) = record_header::<Flags>(bytes)?;
    let (bytes, data) = take(header.size)(bytes)?;
    Ok((bytes, Record { header, data: data.to_vec() }))
}

pub fn record_header<Flags>(bytes: &[u8]) -> IResult<&[u8], RecordHeader<Flags>>
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
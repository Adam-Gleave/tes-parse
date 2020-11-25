use crate::components::*;
use nom::bytes::complete::{tag, take, take_while};
use nom::combinator::map;
use nom::multi::many0;
use nom::number::complete::{
    le_f32,
    le_f64,
    le_i8,
    le_i16,
    le_i32,
    le_i64,
    le_u8,
    le_u16,
    le_u32,
    le_u64,
};
use nom::sequence::tuple;
use nom::IResult;
use std::convert::TryInto;

pub fn parse_plugin(input: &[u8]) -> IResult<&[u8], Plugin> {
    let (remaining, header) = record(input)?;
    let (remaining, top_groups) = many0(group)(remaining)?;
    Ok((remaining, Plugin { header, top_groups }))
}

fn type_code(input: &[u8]) -> IResult<&[u8], TypeCode> {
    let (remaining, code) = take(4usize)(input)?;
    Ok((
        remaining,
        TypeCode {
            code: code.try_into().expect("Parsing type code with incorrect length"),
        },
    ))
}

fn group(input: &[u8]) -> IResult<&[u8], Group> {
    let (remaining, header) = group_header(input)?;
    let (remaining, records_bytes) = take(header.size - 24)(remaining)?;
    let (_, records) = many0(record)(records_bytes)?;
    Ok((remaining, Group { header, records }))
}

fn group_header(input: &[u8]) -> IResult<&[u8], GroupHeader> {
    map(
        tuple((type_code, le_u32, le_u32, le_i32, le_u32, le_u32)),
        |(code, size, label, group_type, vc_info, unknown)| GroupHeader {
            code,
            size,
            label,
            group_type,
            vc_info,
            unknown,
        },
    )(input)
}

fn record(input: &[u8]) -> IResult<&[u8], Record> {
    let (remaining, header) = record_header(input)?;
    let (remaining, subrecords_bytes) = take(header.size)(remaining)?;
    let (_, subrecords) = many0(subrecord)(subrecords_bytes)?;
    Ok((remaining, Record { header, subrecords }))
}

fn record_header(input: &[u8]) -> IResult<&[u8], RecordHeader> {
    map(
        tuple((type_code, le_u32, le_u32, le_u32, le_u32, le_u16, le_u16)),
        |(code, size, flags, id, vc_info, version, unknown)| RecordHeader {
            code,
            size,
            flags,
            id,
            vc_info,
            version,
            unknown,
        },
    )(input)
}

fn subrecord(input: &[u8]) -> IResult<&[u8], Subrecord> {
    let (remaining, header) = subrecord_header(input)?;
    let (remaining, data) = take(header.size)(remaining)?;
    Ok((
        remaining,
        Subrecord {
            header,
            data: data.iter().cloned().collect(),
        },
    ))
}

fn subrecord_header(input: &[u8]) -> IResult<&[u8], SubrecordHeader> {
    map(tuple((type_code, le_u16)), |(code, size)| SubrecordHeader { code, size })(input)
}

fn esp_f32(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_f32(input)?;
    Ok((remaining, EspType::Float32(value)))
}

fn esp_f64(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_f64(input)?;
    Ok((remaining, EspType::Float64(value)))
}

fn esp_i8(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_i8(input)?;
    Ok((remaining, EspType::Int8(value)))
}

fn esp_i16(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_i16(input)?;
    Ok((remaining, EspType::Int16(value)))    
}

fn esp_i32(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_i32(input)?;
    Ok((remaining, EspType::Int32(value)))
}

fn esp_i64(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_i64(input)?;
    Ok((remaining, EspType::Int64(value)))
}

fn esp_u8(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_u8(input)?;
    Ok((remaining, EspType::Uint8(value)))
}

fn esp_u16(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_u16(input)?;
    Ok((remaining, EspType::Uint16(value)))    
}

fn esp_u32(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_u32(input)?;
    Ok((remaining, EspType::Uint32(value)))
}

fn esp_u64(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_u64(input)?;
    Ok((remaining, EspType::Uint64(value)))
}

fn esp_rgb(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, r) = le_u8(input)?;
    let (remaining, g) = le_u8(remaining)?;
    let (remaining, b) = le_u8(remaining)?;
    let (remaining, a) = le_u8(remaining)?;

    Ok((
        remaining,
        EspType::Rgb(
            Rgb { r, g, b, a }
        )
    ))
}

fn esp_formid(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, value) = le_u32(input)?;
    Ok((remaining, EspType::FormID(value)))
}

fn esp_lstring(input: &[u8], localized: bool) -> IResult<&[u8], EspType> {
    let mut lstring = LString::default()
        .with_localized(localized);

    if localized {
        let (remaining, index) = le_u32(input)?;
        lstring = lstring.with_index(index);
        
        Ok((remaining, EspType::LString(lstring)))
    } else {
        let (remaining, content) = take_while(|byte: u8| byte != 0)(input)?;
        let (remaining, _) = tag([0u8])(remaining)?;
        lstring = lstring.with_content(
            std::str::from_utf8(content).unwrap_or("Error decoding string")
        );

        Ok((remaining, EspType::LString(lstring)))
    }
}

fn esp_zstring(input: &[u8]) -> IResult<&[u8], EspType> {
    let (remaining, content) = take_while(|byte: u8| byte != 0)(input)?;
    let (remaining, _) = tag([0u8])(remaining)?;

    Ok((
        remaining, 
        EspType::ZString(
            std::str::from_utf8(content)
                .unwrap_or("Error decoding string")
                .to_owned()
        ),
    ))
}

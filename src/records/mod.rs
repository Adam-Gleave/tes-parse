mod game_setting;
mod unknown;

pub use game_setting::game_setting;
pub use unknown::unknown;

use crate::components::{EspComponent, EspType, EspValue, RecordComponent, Subrecord};
use crate::parser::*;
use nom::bytes::complete::take;
use nom::multi::many0;
use nom::IResult;
use std::collections::HashMap;

fn get_subrecords(data_size: u32, input: &[u8]) -> IResult<&[u8], (Vec<Subrecord>, &[u8])> {
    let (remaining, subrecords_bytes) = take(data_size)(input)?;
    let (_, subrecords) = many0(subrecord)(subrecords_bytes)?;
    Ok((remaining, (subrecords, subrecords_bytes)))
}

fn insert_value_subrecord<'a>(
    code_str: &'a str,
    name: &'a str,
    subrecord_map: &'a mut HashMap<String, Box<dyn EspComponent>>,
    parser: &'a dyn Fn(&[u8]) -> IResult<&[u8], EspType>,
) -> impl FnMut(&[u8]) -> IResult<&[u8], ()> + 'a {
    move |mut subrecords_bytes: &[u8]| {
        let parsed = parser(subrecords_bytes)?;
        let value = parsed.1;

        subrecords_bytes = parsed.0;
        subrecord_map.insert(code_str.to_string(), Box::new(EspValue::new(name, value)));

        Ok((subrecords_bytes, ()))
    }
}

fn insert_variant_subrecord<'a>(
    code_str: &'a str,
    name: &'a str,
    editor_id: &'a str,
    localized: bool,
    subrecord_map: &'a mut HashMap<String, Box<dyn EspComponent>>,
) -> impl FnMut(&[u8]) -> IResult<&[u8], ()> + 'a {
    move |subrecords_bytes: &[u8]| {
        let c = editor_id.chars().next().unwrap().to_ascii_lowercase();
        let (remaining, value) = match c {
            'b' | 'i' => esp_i32(subrecords_bytes)?,
            'f' => esp_f32(subrecords_bytes)?,
            's' => esp_lstring(subrecords_bytes, localized)?,
            _ => panic!(),
        };

        subrecord_map.insert(code_str.to_string(), Box::new(EspValue::new(name, value)));
        Ok((remaining, ()))
    }
}

pub fn common_record<'a, F>(name: &str, f: F, input: &'a [u8]) -> IResult<&'a [u8], RecordResult>
where
    F: Fn(&str, &'a [u8], &mut HashMap<String, Box<dyn EspComponent>>) -> IResult<&'a [u8], ()>,
{
    let (remaining, header) = record_header(input)?;
    let mut subrecord_map: HashMap<String, Box<dyn EspComponent>> = HashMap::new();
    let (remaining, (subrecords, mut subrecords_bytes)) = get_subrecords(header.size, remaining)?;

    for subrecord in subrecords.iter() {
        subrecords_bytes = take(6usize)(subrecords_bytes)?.0;
        let code_str = subrecord
            .header
            .code
            .to_utf8()
            .expect(&format!("Error parsing subrecord code {:#?}", subrecord.header.code));

        subrecords_bytes = f(code_str, subrecords_bytes, &mut subrecord_map)?.0;
    }

    Ok((
        remaining,
        RecordResult::Single(Box::new(RecordComponent::new(name, subrecord_map))),
    ))
}

pub fn color_record<'a>(name: &str, input: &'a [u8]) -> IResult<&'a [u8], RecordResult> {
    let f = |code_str: &str,
             subrecords_bytes: &'a [u8],
             subrecord_map: &mut HashMap<String, Box<dyn EspComponent>>|
     -> IResult<&'a [u8], ()> {
        let subrecords_bytes = match code_str {
            "EDID" => insert_value_subrecord("EDID", "Editor ID", subrecord_map, &esp_zstring)(subrecords_bytes)?.0,
            "CNAM" => insert_value_subrecord("CNAM", "Color", subrecord_map, &esp_rgb)(subrecords_bytes)?.0,
            _ => subrecords_bytes,
        };

        Ok((subrecords_bytes, ()))
    };

    let (remaining, record) = common_record(name, f, input)?;
    Ok((remaining, record))
}

mod game_setting;
mod unknown;

pub use unknown::unknown;
pub use game_setting::game_setting;

use crate::parser::*;
use crate::components::{EspComponent, EspType, EspValue, RecordComponent, Subrecord};
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
    subrecord_map: &'a mut std::collections::HashMap<String, Box<dyn EspComponent>>,
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
    subrecord_map: &'a mut std::collections::HashMap<String, Box<dyn EspComponent>>,
) -> impl FnMut(&[u8]) -> IResult<&[u8], ()> + 'a {
    move |subrecords_bytes: &[u8]| {
        let c = editor_id.chars().next().unwrap().to_ascii_lowercase();
        let (remaining, value) = match c {
            'b' | 'i' => esp_i32(subrecords_bytes)?,
            'f' => esp_f32(subrecords_bytes)?,
            's' => esp_lstring(subrecords_bytes, localized)?,
            _ => { panic!() },
        };

        subrecord_map.insert(code_str.to_string(), Box::new(EspValue::new(name, value)));
        Ok((remaining, ()))
    }
}

pub fn color_record<'a>(name: &str, input: &'a [u8]) -> IResult<&'a [u8], RecordResult> {
    let (remaining, header) = record_header(input)?;
    let mut subrecord_map: HashMap<String, Box<dyn EspComponent>> = HashMap::new();
    let (remaining, (subrecords, mut subrecords_bytes)) = get_subrecords(header.size, remaining)?;

    for subrecord in subrecords.iter() {
        subrecords_bytes = take(6usize)(subrecords_bytes)?.0;
        let code_str = subrecord.header.code
            .to_utf8()
            .expect(&format!("Error parsing subrecord code {:#?}", subrecord.header.code));

        subrecords_bytes = match code_str {
            "EDID" => { insert_value_subrecord("EDID", "Editor ID", &mut subrecord_map, &esp_zstring)(subrecords_bytes)?.0 },
            "CNAM" => { insert_value_subrecord("CNAM", "Color", &mut subrecord_map, &esp_rgb)(subrecords_bytes)?.0 },
            _ => { subrecords_bytes },
        }
    }

    Ok((
        remaining, 
        RecordResult::Single(Box::new(
            RecordComponent::new(name, subrecord_map)
        )),
    ))
}

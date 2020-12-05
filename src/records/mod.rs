mod unknown;

pub use unknown::unknown;

use crate::parser::*;
use crate::components::{EspComponent, EspType, EspValue, RecordComponent, Subrecord};
use nom::bytes::complete::take;
use nom::multi::many0;
use nom::IResult;
use std::collections::HashMap;

fn get_subrecords(input: &[u8]) -> IResult<&[u8], (Vec<Subrecord>, &[u8])> {
    let (remaining, header) = record_header(input)?;
    let (remaining, subrecords_bytes) = take(header.size)(remaining)?;
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

pub fn color_record<'a>(name: &str, input: &'a [u8]) -> IResult<&'a [u8], RecordResult> {
    let mut subrecord_map: HashMap<String, Box<dyn EspComponent>> = HashMap::new();
    let (remaining, (subrecords, mut subrecords_bytes)) = get_subrecords(input)?;

    for subrecord in subrecords.iter() {
        let code_str = subrecord.header.code
            .to_utf8()
            .expect(&format!("Error parsing subrecord code {:#?}", subrecord.header.code));

        match code_str {
            "EDID" => { subrecords_bytes = insert_value_subrecord("EDID", "Editor ID", &mut subrecord_map, &esp_zstring)(subrecords_bytes)?.0 },
            "CNAM" => { subrecords_bytes = insert_value_subrecord("CNAM", "Color", &mut subrecord_map, &esp_rgb)(subrecords_bytes)?.0 },
            _ => {},
        }
    }

    Ok((
        remaining, 
        RecordResult::Single(Box::new(
            RecordComponent::new(name, subrecord_map)
        )),
    ))
}

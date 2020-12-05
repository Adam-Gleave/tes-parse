use crate::components::*;
use crate::parser::*;
use nom::bytes::complete::take;
use nom::multi::many0;
use nom::IResult;
use std::collections::HashMap;

pub fn keyword(input: &[u8]) -> IResult<&[u8], RecordComponent> {
    let mut subrecord_map: HashMap<String, Box<dyn EspComponent>> = HashMap::new();
    let (remaining, header) = record_header(input)?;
    let (remaining, mut subrecords_bytes) = take(header.size)(remaining)?;
    let (_, subrecords) = many0(subrecord)(subrecords_bytes)?;

    for subrecord in subrecords.iter() {
        let code_str = subrecord.header.code
            .to_utf8()
            .expect(&format!("Error parsing subrecord code {:#?}", subrecord.header.code));

        match code_str {
            "EDID" => { 
                let parsed = esp_zstring(subrecords_bytes)?;
                let value = parsed.1;
                subrecords_bytes = parsed.0;

                subrecord_map.insert(
                    "EDID".to_string(), 
                    Box::new(EspValue::new("Editor ID", value)),
                ); 
            },
            "CNAM" => {
                let parsed = esp_rgb(subrecords_bytes)?;
                let value = parsed.1;
                subrecords_bytes = parsed.0;

                subrecord_map.insert(
                    "CNAM".to_string(),
                    Box::new(EspValue::new("Color", value)),
                );
            },
            _ => {},
        }
    }

    Ok((
        remaining, 
        RecordComponent::new("Keyword", subrecord_map),
    ))
}

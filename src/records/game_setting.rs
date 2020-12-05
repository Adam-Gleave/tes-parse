use crate::components::*;
use crate::parser::*;
use super::*;
use nom::bytes::complete::take;
use nom::IResult;
use std::collections::HashMap;

pub fn game_setting(input: &[u8]) -> IResult<&[u8], RecordResult> {
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
            "DATA" => {
                let editor_id = (*(subrecord_map.get("EDID").unwrap().get("").unwrap())).clone();

                if let EspType::ZString(editor_id) = &editor_id.value {
                    insert_variant_subrecord("DATA", "Data", &editor_id, true, &mut subrecord_map)(subrecords_bytes)?.0
                } else {
                    subrecords_bytes
                }
            },
            _ => { subrecords_bytes },
        };
    }

    Ok((
        remaining, 
        RecordResult::Single(Box::new(
            RecordComponent::new("Game Setting", subrecord_map)
        )),
    ))
}
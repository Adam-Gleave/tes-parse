use super::*;
use crate::components::*;
use crate::parser::*;
use nom::IResult;
use std::collections::HashMap;

pub fn game_setting<'a>(input: &'a [u8]) -> IResult<&'a [u8], RecordResult> {
    let f = |code_str: &str,
             subrecords_bytes: &'a [u8],
             subrecord_map: &mut HashMap<String, Box<dyn EspComponent>>|
     -> IResult<&'a [u8], ()> {
        let subrecords_bytes = match code_str {
            "EDID" => {
                insert_value_subrecord("EDID", "Editor ID", subrecord_map, &esp_zstring)(
                    subrecords_bytes,
                )?
                .0
            }
            "DATA" => {
                let editor_id = (*(subrecord_map.get("EDID").unwrap().get("").unwrap())).clone();

                if let EspType::ZString(editor_id) = &editor_id.value {
                    insert_variant_subrecord("DATA", "Data", &editor_id, true, subrecord_map)(
                        subrecords_bytes,
                    )?
                    .0
                } else {
                    subrecords_bytes
                }
            }
            _ => subrecords_bytes,
        };

        Ok((subrecords_bytes, ()))
    };

    let (remaining, record) = common_record("Game Setting", f, input)?;
    Ok((remaining, record))
}

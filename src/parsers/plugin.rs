use std::collections::HashMap;

use crate::parsers::{common::TypeCode, group, records};

#[derive(Debug)]
pub struct Plugin {
    pub tes4: records::FileHeaderRecord,
    pub groups: HashMap<TypeCode, group::Group>,
}

impl Plugin {
    pub fn get_editor_ids_by_code(&self, code: [u8; 4]) -> Vec<String> {
        let code: TypeCode = code.into();

        if let Some(group) = self.groups.get(&code) {
            if let group::GroupData::Records(records) = &group.data {
                let mut editor_ids = vec![];

                for record in records {
                    if let Some(editor_id) = &record.1.header.editor_id {
                        editor_ids.push(editor_id.clone());
                    }
                }

                editor_ids
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

pub fn plugin(bytes: &[u8]) -> crate::IResult<&[u8], Plugin> {
    let (mut bytes, tes4) = records::file_header_record(bytes)?;
    let mut groups = HashMap::new();

    while bytes.len() > 0 {
        let (remaining, (code, group)) = group::top_group(bytes)?;
        groups.insert(code, group);
        bytes = remaining;
    }

    Ok((bytes, Plugin { tes4, groups }))
}

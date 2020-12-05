use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

pub struct Plugin {
    pub header: Box<dyn EspComponent>,
    pub top_groups: Vec<Group>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypeCode {
    pub code: [u8; 4],
}

impl fmt::Display for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = std::str::from_utf8(&self.code[..]).unwrap().to_owned();
        write!(f, "{}", code)
    }
}

impl From<u32> for TypeCode {
    fn from(input: u32) -> Self {
        Self {
            code: unsafe { std::mem::transmute(input.to_le()) },
        }
    }
}

impl Into<u32> for TypeCode {
    fn into(self) -> u32 {
        unsafe { std::mem::transmute(self.code) }
    }
}

impl TypeCode {
    pub fn from_utf8(input: &str) -> Result<Self, std::io::Error> {
        let bytes = input.as_bytes();

        if let Ok(code_byte_arr) = bytes.try_into() {
            Ok(Self {
                code: code_byte_arr,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Type code string is required to be 4 characters long",
            ))
        }
    }

    pub fn to_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.code)
    }
}

pub enum GroupType {
    Top = 0,
    WorldChildren = 1,
    InteriorCellBlock = 2,
    InteriorSubCellBlock = 3,
    ExteriorCellBlock = 4,
    ExteriorSubCellBlock = 5,
    CellChildren = 6,
    TopicChildren = 7,
    CellPersistenChildren = 8,
    CellTemporaryChildren = 9,
}

pub struct Group {
    pub header: GroupHeader,
    pub records: Vec<Box<dyn EspComponent>>,
}

impl Group {
    pub fn type_code(&self) -> Option<TypeCode> {
        // Make sure this is a top group
        if self.header.group_type == GroupType::Top as i32 {
            Some(self.header.label.into())
        } else {
            None
        }
    }

    pub fn top_group_matches_type(&self, code: TypeCode) -> bool {
        if let Some(group_type_code) = self.type_code() {
            code == group_type_code
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GroupHeader {
    pub code: TypeCode,
    pub size: u32,
    pub label: u32,
    pub group_type: i32,
    pub vc_info: u32,
    pub unknown: u32,
}

pub struct RecordComponent {
    name: String,
    subrecords: HashMap<String, Box<dyn EspComponent>>,
}

impl RecordComponent {
    pub fn new(name: &str, subrecords: HashMap<String, Box<dyn EspComponent>>) -> Self {
        Self {
            name: name.to_owned(),
            subrecords,
        }
    }
}

impl EspComponent for RecordComponent {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> EspComponentType {
        EspComponentType::Record
    }

    fn get(&self, accessor: &str) -> Option<&EspValue> {
        let accessor = accessor.to_owned();

        if let Some(split_index) = accessor.find("/") {
            let (subrecord_code, subrecord_accessor) = accessor.split_at(split_index);

            if let Some(subrecord) = self.subrecords.get(subrecord_code) {
                subrecord.get(subrecord_accessor)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue> {
        let accessor = accessor.to_owned();

        if let Some(split_index) = accessor.find("/") {
            let (subrecord_code, subrecord_accessor) = accessor.split_at(split_index);

            if let Some(subrecord) = self.subrecords.get_mut(subrecord_code) {
                subrecord.get_mut(subrecord_accessor)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub header: RecordHeader,
    pub subrecords: Vec<Subrecord>,
}

impl Record {
    pub fn editor_id(&self) -> Option<String> {
        if self.subrecords.is_empty() {
            None
        } else {
            let edid_subrecord = self.subrecords.iter().find(|s| {
                return s.header.code == TypeCode::from_utf8("EDID").unwrap();
            });

            if let Some(edid) = edid_subrecord {
                Some(String::from_utf8(edid.data.clone()).unwrap())
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordHeader {
    pub code: TypeCode,
    pub size: u32,
    pub flags: u32,
    pub id: u32,
    pub vc_info: u32,
    pub version: u16,
    pub unknown: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subrecord {
    pub header: SubrecordHeader,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubrecordHeader {
    pub code: TypeCode,
    pub size: u16,
}

pub enum EspComponentType {
    Value,
    Array,
    Struct,
    Record,
}

pub trait EspComponent: Send + Sync {
    fn name(&self) -> &str;
    fn component_type(&self) -> EspComponentType;
    fn get(&self, accessor: &str) -> Option<&EspValue>;
    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue>;
}

#[derive(Debug, Clone)]
pub struct EspValue {
    name: String,
    pub value: EspType,
}

impl EspValue {
    pub fn new(name: &str, value: EspType) -> Self {
        Self {
            name: name.to_owned(),
            value,
        }
    }
}

impl EspComponent for EspValue {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> EspComponentType {
        EspComponentType::Value
    }

    fn get(&self, accessor: &str) -> Option<&EspValue> {
        Some(self)
    }

    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue> {
        Some(self)
    }
}

#[derive(Debug)]
pub struct EspArray {
    name: String,
    array: Vec<EspValue>,
}

impl EspComponent for EspArray {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> EspComponentType {
        EspComponentType::Array
    }

    fn get(&self, accessor: &str) -> Option<&EspValue> {
        self.array.get(0)
    }

    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue> {
        self.array.get_mut(0)
    }
}

#[derive(Debug)]
pub struct EspStruct {
    name: String,
    fields: HashMap<String, EspValue>,
}

impl EspComponent for EspStruct {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> EspComponentType {
        EspComponentType::Struct
    }

    fn get(&self, accessor: &str) -> Option<&EspValue> {
        self.fields.get("")
    }

    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue> {
        self.fields.get_mut("")
    }
}

#[derive(Debug, Clone)]
pub enum EspType {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Float32(f32),
    Float64(f64),
    Rgb(Rgb),
    FormID(u32),
    ZString(String),
    LString(LString),
}

#[derive(Debug, Default, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone)]
pub struct LString {
    index: u32,
    content: String,
    modified: bool,
    localized: bool,
}

impl Default for LString {
    fn default() -> Self {
        Self {
            index: 0,
            content: "Unknown LString".to_owned(),
            modified: false,
            localized: false,
        }
    }
}

impl LString {
    pub fn with_index(self, index: u32) -> Self {
        Self {
            index,
            content: self.content,
            modified: self.modified,
            localized: self.localized,
        }
    }

    pub fn with_content(self, content: &str) -> Self {
        Self {
            index: self.index,
            content: content.to_owned(),
            modified: self.modified,
            localized: self.localized,
        }
    }

    pub fn with_localized(self, localized: bool) -> Self {
        Self {
            index: self.index,
            content: self.content,
            modified: self.modified,
            localized,
        }
    }

    pub fn set(&mut self, str: &str) {
        self.content = str.to_owned();
        self.modified = true;
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }
}

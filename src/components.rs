use std::collections::HashMap;

pub enum EspComponentType {
    Value,
    Array,
    Struct,
}

pub trait EspComponent {
    fn name(&self) -> &str;
    fn component_type() -> EspComponentType;
    fn get(&self, accessor: &str) -> Option<&EspValue>;
    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue>;
}

#[derive(Debug)]
pub struct EspValue {
    name: String,
    pub value: EspType,
}

impl EspComponent for EspValue {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type() -> EspComponentType {
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

    fn component_type() -> EspComponentType {
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

    fn component_type() -> EspComponentType {
        EspComponentType::Struct
    }

    fn get(&self, accessor: &str) -> Option<&EspValue> {
        self.fields.get("")
    }

    fn get_mut(&mut self, accessor: &str) -> Option<&mut EspValue> {
        self.fields.get_mut("")
    }
}

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug)]
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
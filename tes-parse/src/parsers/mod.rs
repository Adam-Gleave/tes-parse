use crate::components::*;
use crate::parser::*;
use tes_parse_derive::ValueParser;

pub trait Context<'a> {
    type Flags;

    fn get_flags(&self) -> Self::Flags;
    fn get_bytes(&self) -> &[u8];
    fn set_bytes(&mut self, input: &'a [u8]);
}

pub(crate) trait Parser<'a> {
    type Output;
    type Flags;

    fn do_parse(&self, ctx: &'a mut dyn Context<'a, Flags = Self::Flags >) -> Self::Output;
}

pub struct PluginContext<'a> {
    pub tes4_header: RecordHeader,
    pub bytes: &'a [u8],
}

impl<'a> Context<'a> for PluginContext<'a> {
    type Flags = u32;
    
    fn get_flags(&self) -> u32 {
        self.tes4_header.flags
    }

    fn get_bytes(&self) -> &[u8] {
        self.bytes
    }

    fn set_bytes(&mut self, input: &'a [u8]) {
        self.bytes = input;
    }
}

pub(crate) struct PluginParser<'a> {
    header_parser: Box<dyn Parser<'a, Output = RecordResult, Flags = u32> + 'a>,
    group_parsers: Vec<Box<dyn Parser<'a, Output = Group, Flags = u32> + 'a>>,
}

impl<'a> PluginParser<'a> {
    pub fn with_group(mut self, parser: Box<dyn Parser<'a, Output = Group, Flags = u32>>) -> Self {
        self.group_parsers.push(parser);
        self
    }
}

impl<'a> Parser<'a> for PluginParser<'a> {
    type Output = Plugin;
    type Flags = u32;

    fn do_parse(&self, ctx: &'a mut dyn Context<Flags = Self::Flags>) -> Self::Output {
        Plugin {
            
        }
    }
}

pub struct GroupParser<'a> {
    record_parser: Box<dyn Parser<'a, Output = RecordResult, Flags = u32> + 'a>,
}

impl<'a> GroupParser<'a> {
    pub fn with_record(mut self, parser: Box<dyn Parser<'a, Output = RecordResult, Flags = u32>>) -> Self {
        self.record_parser = parser;
        self
    }
}

impl<'a> Parser<'a> for GroupParser<'a> {
    type Output = Group;
    type Flags = u32;

    fn do_parse(&self, ctx: &'a mut dyn Context<Flags = Self::Flags>) -> Self::Output {
        Group {

        }
    }
}

pub struct RecordParser<'a> {
    subrecord_parsers: Vec<Box<dyn Parser<'a, Output = Subrecord, Flags = u32> + 'a>>,
}

impl<'a> RecordParser<'a> {
    pub fn with_subrecord(mut self, parser: Box<dyn Parser<'a, Output = Subrecord, Flags = u32>>) -> Self {
        self.subrecord_parsers.push(parser);
        self
    }
}

impl<'a> Parser<'a> for RecordParser<'a> {
    type Output = RecordResult;
    type Flags = u32;

    fn do_parse(&self, ctx: &'a mut dyn Context<Flags = Self::Flags>) -> Self::Output {
        RecordResult {

        }
    }
}

pub struct ValueSubrecordParser<'a> {
    code: TypeCode,
    name: String,
    value_parser: Box<dyn Parser<'a, Output = EspValue, Flags = u32> + 'a>,
}

impl<'a> ValueSubrecordParser<'a> {
    pub fn with_code(mut self, code: &str) -> Self {
        self.code = TypeCode::from_utf8(code).expect(&format!("Cannot parse subrecord code: {}", code));
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_value(mut self, parser: Box<dyn Parser<'a, Output = EspValue, Flags = u32>>) -> Self {
        self.value_parser = parser;
        self
    }
}

impl<'a> Parser<'a> for ValueSubrecordParser<'a> {
    type Output = Subrecord;
    type Flags = u32;

    fn do_parse(&self, ctx: &'a mut dyn Context<Flags = Self::Flags>) -> Self::Output {
        Subrecord {

        }
    }
}

#[derive(ValueParser)]
#[parse_fn(esp_u8)]
pub struct Uint8Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_u16)]
pub struct Uint16Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_u32)]
pub struct Uint32Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_u64)]
pub struct Uint64Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_i8)]
pub struct Int8Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_i16)]
pub struct Int16Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_i32)]
pub struct Int32Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_i64)]
pub struct Int64Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_f32)]
pub struct Float32Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_f64)]
pub struct Float64Parser {}

#[derive(ValueParser)]
#[parse_fn(esp_rgb)]
pub struct RgbParser {}

#[derive(ValueParser)]
#[parse_fn(esp_formid)]
pub struct FormIDParser {}

#[derive(ValueParser)]
#[parse_fn(esp_lstring)]
pub struct LStringParser {}

#[derive(ValueParser)]
#[parse_fn(esp_zstring)]
pub struct ZStringParser {}

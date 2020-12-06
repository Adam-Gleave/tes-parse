use crate::components::*;
use crate::parser::*;
use crate::records::*;
use nom::bytes::complete::take;
use nom::multi::many0;
use nom::IResult;
use std::cell::RefCell;
use std::collections::HashMap;
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

    fn code(&self) -> &str;
    fn do_parse(&self, ctx: &'a mut dyn Context<'a, Flags = Self::Flags>) -> Option<Self::Output>;
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

    fn code(&self) -> &str {
        "PLUGIN"
    }

    fn do_parse(&self, ctx: &'a mut dyn Context<Flags = Self::Flags>) -> Option<Self::Output> {
        None
    }
}

pub struct GroupParser<'a> {
    record_parser: Box<dyn Parser<'a, Output = RecordResult, Flags = u32> + 'a>,
}

impl<'a> GroupParser<'a> {
    pub fn with_record(
        mut self,
        parser: Box<dyn Parser<'a, Output = RecordResult, Flags = u32>>,
    ) -> Self {
        self.record_parser = parser;
        self
    }
}

impl<'a> Parser<'a> for GroupParser<'a> {
    type Output = Group;
    type Flags = u32;

    fn code(&self) -> &str {
        "GRUP"
    }

    fn do_parse(&self, ctx: &'a mut dyn Context<Flags = Self::Flags>) -> Option<Self::Output> {
        None
    }
}

pub struct RecordParser<'a> {
    code: TypeCode,
    name: String,
    subrecord_parsers:
        RefCell<HashMap<String, Box<dyn Parser<'a, Output = EspValue, Flags = u32> + 'a>>>,
}

impl<'a> RecordParser<'a> {
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: TypeCode::from_utf8(code).expect(&format!("Cannot parse record code: {}", code)),
            name: name.to_owned(),
            subrecord_parsers: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_subrecord(
        mut self,
        parser: Box<dyn Parser<'a, Output = EspValue, Flags = u32>>,
    ) -> Self {
        self.subrecord_parsers
            .borrow_mut()
            .insert(parser.code().to_owned(), parser);
        self
    }

    fn get_subrecords(data_size: u32, input: &[u8]) -> IResult<&[u8], (Vec<Subrecord>, &[u8])> {
        let (remaining, subrecords_bytes) = take(data_size)(input)?;
        let (_, subrecords) = many0(subrecord)(subrecords_bytes)?;
        Ok((remaining, (subrecords, subrecords_bytes)))
    }
}

impl<'a> Parser<'a> for RecordParser<'a> {
    type Output = RecordResult;
    type Flags = u32;

    fn code(&self) -> &str {
        self.code.to_utf8().unwrap()
    }

    fn do_parse(&self, ctx: &'a mut dyn Context<'a, Flags = Self::Flags>) -> Option<Self::Output> {
        let (remaining, header) = record_header(ctx.get_bytes()).unwrap();
        let (remaining, (subrecords, mut subrecords_bytes)) =
            get_subrecords(header.size, remaining).unwrap();

        let mut parsed_subrecords = vec![];

        for subrecord in subrecords.iter() {
            let parsed: IResult<&[u8], &[u8]> = take(6usize)(subrecords_bytes);
            let subrecords_bytes = parsed.unwrap().0;

            let code_str = subrecord.header.code.to_utf8().expect(&format!(
                "Cannot parse subrecord code: {:#?}",
                subrecord.header.code
            ));

            let subrecord_parser = self
                .subrecord_parsers
                .borrow()
                .get(code_str)
                .expect(&format!("No parser for {} subrecord", code_str));

            let subrecord = subrecord_parser.do_parse(ctx);
            parsed_subrecords.push(subrecord);
        }

        None
    }
}

pub struct ValueSubrecordParser<'a> {
    pub code: TypeCode,
    pub name: String,
    value_parser: Option<Box<dyn Parser<'a, Output = EspValue, Flags = u32> + 'a>>,
}

impl<'a> ValueSubrecordParser<'a> {
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: TypeCode::from_utf8(code)
                .expect(&format!("Cannot parse subrecord code: {}", code)),
            name: name.to_owned(),
            value_parser: None,
        }
    }

    pub fn with_value(
        mut self,
        parser: Box<dyn Parser<'a, Output = EspValue, Flags = u32>>,
    ) -> Self {
        self.value_parser = Some(parser);
        self
    }
}

impl<'a> Parser<'a> for ValueSubrecordParser<'a> {
    type Output = Box<dyn EspComponent>;
    type Flags = u32;

    fn code(&self) -> &str {
        self.code.to_utf8().unwrap()
    }

    fn do_parse(&self, ctx: &'a mut dyn Context<'a, Flags = Self::Flags>) -> Option<Self::Output> {
        let (remaining, header) = subrecord_header(ctx.get_bytes()).expect("Parser error");

        let (remaining, value) = if let Some(parser) = self.value_parser {
            let value = parser.do_parse(ctx).unwrap();
            (
                ctx.get_bytes(),
                Some(Box::new(value) as Box<dyn EspComponent>),
            )
        } else {
            let parsed: IResult<&[u8], _> = take(header.size as usize)(remaining);
            (parsed.unwrap().0, None)
        };

        ctx.set_bytes(remaining);
        value
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

use super::prelude::*;
use super::{
    common::{FormId, TypeCode},
    records::{record, Record},
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Group {
    pub size: u32,
    pub label: Label,
    pub group_type: GroupType,
    pub timestamp: u16,
    pub vc_info: u16,
    pub data: GroupData,
}

impl Group {
    pub const CODE: TypeCode = TypeCode([b'G', b'R', b'U', b'P']);
    pub const HEADER_SIZE: usize = 24;
}

pub(super) fn group(bytes: &[u8]) -> IResult<&[u8], Group> {
    let (bytes, mut group): (&[u8], Group) = map(
        delimited(
            take(4usize),
            tuple((le_u32, take(4usize), take(4usize), le_u16, le_u16)),
            take(4usize),
        ),
        |(size, label, mut group_type, timestamp, vc_info): (u32, &[u8], &[u8], u16, u16)| {
            let group_type = group_type.read_i32::<LittleEndian>().unwrap().into();
            let label = label_given_type(label, &group_type);

            Group {
                size,
                label,
                group_type,
                timestamp,
                vc_info,
                data: GroupData::Unimplemented(Vec::<u8>::new()),
            }
        },
    )(bytes)?;

    let (bytes, group_data) = group_data(bytes, group.group_type, &group.label, group.size)?;
    group.data = group_data;

    Ok((bytes, group))
}

pub(super) fn top_group(bytes: &[u8]) -> IResult<&[u8], (TypeCode, Group)> {
    let (bytes, group) = group(bytes)?;

    if let Label::RecordType(code) = group.label.clone() {
        Ok((bytes, (code, group)))
    } else {
        panic!("Top group does not have a TypeCode label");
    }
}

fn label_given_type(bytes: &[u8], group_type: &GroupType) -> Label {
    match *group_type {
        GroupType::Top => {
            let arr: [u8; 4] = bytes.try_into().unwrap();
            Label::RecordType(arr.into())
        }
        GroupType::WorldChildren => Label::ParentWorld(form_id_from_vec(bytes)),
        GroupType::InteriorCellBlock => Label::BlockNumber(i32_from_vec(bytes).into()),
        GroupType::InteriorCellSubBlock => Label::SubBlockNumber(i32_from_vec(bytes).into()),
        GroupType::ExteriorCellBlock => Label::GridCoordinate(grid_coord_from_vec(bytes)),
        GroupType::ExteriorCellSubBlock => Label::GridCoordinate(grid_coord_from_vec(bytes)),
        GroupType::CellChildren => Label::ParentCell(form_id_from_vec(bytes)),
        GroupType::TopicChildren => Label::ParentDialog(form_id_from_vec(bytes)),
        GroupType::CellPersistenChildren => Label::ParentCell(form_id_from_vec(bytes)),
        GroupType::CellTemporaryChildren => Label::ParentCell(form_id_from_vec(bytes)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupType {
    Top = 0,
    WorldChildren = 1,
    InteriorCellBlock = 2,
    InteriorCellSubBlock = 3,
    ExteriorCellBlock = 4,
    ExteriorCellSubBlock = 5,
    CellChildren = 6,
    TopicChildren = 7,
    CellPersistenChildren = 8,
    CellTemporaryChildren = 9,
}

impl From<i32> for GroupType {
    fn from(val: i32) -> Self {
        match val {
            0 => Self::Top,
            1 => Self::WorldChildren,
            2 => Self::InteriorCellBlock,
            3 => Self::InteriorCellSubBlock,
            4 => Self::ExteriorCellBlock,
            5 => Self::ExteriorCellSubBlock,
            6 => Self::CellChildren,
            7 => Self::TopicChildren,
            8 => Self::CellPersistenChildren,
            9 => Self::CellTemporaryChildren,
            _ => panic!("Invalid group type {}", val),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Label {
    BlockNumber(i32),
    GridCoordinate([u16; 2]),
    ParentCell(FormId),
    ParentDialog(FormId),
    ParentWorld(FormId),
    RecordType(TypeCode),
    SubBlockNumber(i32),
}

#[derive(Debug)]
pub enum GroupData {
    Records(Vec<(String, Record)>),
    Unimplemented(Vec<u8>),
}

fn group_data<'a>(bytes: &'a [u8], group_type: GroupType, label: &Label, size: u32) -> IResult<&'a [u8], GroupData> {
    let (remaining, mut group_bytes) = take(size as usize - Group::HEADER_SIZE)(bytes)?;

    match group_type {
        GroupType::Top => match label {
            Label::RecordType(code) => match code.to_string().as_str() {
                "CELL" | "WRLD" | "DIAL" => Ok((remaining, GroupData::Unimplemented(group_bytes.to_vec()))),
                _ => {
                    let mut records = Vec::new();

                    while group_bytes.len() > 0 {
                        let (group_remaining, record) = record(group_bytes)?;
                        group_bytes = group_remaining;
                        records.push(record);
                    }

                    Ok((remaining, GroupData::Records(records)))
                }
            },
            _ => Ok((remaining, GroupData::Unimplemented(group_bytes.to_vec()))),
        },
        _ => Ok((remaining, GroupData::Unimplemented(group_bytes.to_vec()))),
    }
}

fn form_id_from_vec(mut v: &[u8]) -> FormId {
    v.read_u32::<LittleEndian>().unwrap().into()
}

fn i32_from_vec(mut v: &[u8]) -> i32 {
    v.read_i32::<LittleEndian>().unwrap()
}

fn grid_coord_from_vec(mut v: &[u8]) -> [u16; 2] {
    [v.read_u16::<LittleEndian>().unwrap(), v.read_u16::<LittleEndian>().unwrap()]
}

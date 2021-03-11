use super::prelude::*;
use crate::IResult;
use std::fmt;
use std::ops::{self, Deref};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TypeCode(pub(crate) [u8; 4]);

impl fmt::Debug for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code_str = std::str::from_utf8(self.deref()).or(Err(fmt::Error))?;
        write!(f, "{}", code_str)
    }
}

impl fmt::Display for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code_str = std::str::from_utf8(self.deref()).or(Err(fmt::Error))?;
        write!(f, "{}", code_str)
    }
}

impl From<[u8; 4]> for TypeCode {
    fn from(code: [u8; 4]) -> Self {
        Self(code)
    }
}

impl From<u32> for TypeCode {
    fn from(code: u32) -> Self {
        Self(code.to_le_bytes())
    }
}

impl ops::Deref for TypeCode {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Clone)]
pub struct FormId(u32);

impl From<u32> for FormId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl ops::Deref for FormId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for FormId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#010X}", self.deref())
    }
}

impl fmt::Display for FormId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#010X}", self.deref())
    }
}

pub(super) fn form_id(bytes: &[u8]) -> IResult<&[u8], FormId> {
    map(le_u32, |id| id.into())(bytes)
}

pub(super) fn zstring(bytes: &[u8]) -> IResult<&[u8], String> {
    map(
        terminated(take_while(|c| c != 0), tag([0u8])),
        |zstring_bytes: &[u8]| String::from_utf8(zstring_bytes.to_vec()).unwrap(),
    )(bytes)
}

pub(super) fn subrecords(bytes: &[u8]) -> IResult<&[u8], Vec<(TypeCode, &[u8])>> {
    many0(pair(
        map(le_u32, |code| TypeCode::from(code)),
        flat_map(le_u16, take),
    ))(bytes)
}

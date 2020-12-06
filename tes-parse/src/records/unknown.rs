use crate::components::*;
use crate::parser::*;
use nom::bytes::complete::take;
use nom::IResult;
use std::collections::HashMap;

pub fn unknown(input: &[u8]) -> IResult<&[u8], RecordResult> {
    let (remaining, header) = record_header(input)?;
    let (remaining, _) = take(header.size)(remaining)?;

    Ok((
        remaining,
        RecordResult::Single(Box::new(RecordComponent::new("Unknown", HashMap::new()))),
    ))
}

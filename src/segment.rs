use winnow::binary::{be_u16, be_u32, be_u8, length_take};
use winnow::token::literal;
use winnow::{Bytes, PResult, Parser};

#[derive(Debug, Clone)]
pub struct Segment<'a> {
    pub pts: u32,
    pub dts: u32,
    pub kind: u8,
    pub body: &'a Bytes,
}

pub fn segment<'a>(input: &mut &'a Bytes) -> PResult<Segment<'a>> {
    literal(b"PG").parse_next(input)?;
    Ok(Segment {
        pts: be_u32.parse_next(input)?,
        dts: be_u32.parse_next(input)?,
        kind: be_u8.parse_next(input)?,
        body: Bytes::new(length_take(be_u16).parse_next(input)?),
    })
}

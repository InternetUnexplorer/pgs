use winnow::binary::{be_u16, be_u24, be_u8};
use winnow::combinator::{alt, repeat, rest};
use winnow::token::literal;
use winnow::{Bytes, PResult, Parser};

#[derive(Debug, Clone)]
pub struct Ods<'a> {
    pub id: u16,
    pub version: u8,
    pub sequence: u8,
    pub body: &'a Bytes,
}

impl Ods<'_> {
    pub fn is_first_in_sequence(&self) -> bool {
        self.sequence & 0x80 == 0x80
    }
    pub fn is_last_in_sequence(&self) -> bool {
        self.sequence & 0x40 == 0x40
    }
    pub fn is_first_and_last_in_sequence(&self) -> bool {
        self.is_first_in_sequence() && self.is_last_in_sequence()
    }
}

pub fn ods<'a>(input: &mut &'a Bytes) -> PResult<Ods<'a>> {
    Ok(Ods {
        id: be_u16.parse_next(input)?,
        version: be_u8.parse_next(input)?,
        sequence: be_u8.parse_next(input)?,
        body: Bytes::new(rest.parse_next(input)?),
    })
}

pub fn rle(input: &mut &Bytes) -> PResult<(u16, u16, Vec<(u16, u8)>)> {
    let _length = be_u24.parse_next(input)? as usize;
    Ok((
        be_u16.parse_next(input)?,
        be_u16.parse_next(input)?,
        repeat(0.., rle_word).parse_next(input)?,
    ))
}

fn rle_word(input: &mut &Bytes) -> PResult<(u16, u8)> {
    match be_u8.parse_next(input)? {
        0 => (),
        c => return Ok((1, c)),
    };

    fn word_6b0(input: &mut &Bytes) -> PResult<(u16, u8)> {
        let l = be_u8.verify(|&l| l <= 63).parse_next(input)?;
        Ok((l as u16, 0))
    }

    fn word_14b0(input: &mut &Bytes) -> PResult<(u16, u8)> {
        let l = be_u16.verify(|&l| l >= 16448 && l <= 32767).parse_next(input)? & 16383;
        Ok((l as u16, 0))
    }

    fn word_6bc(input: &mut &Bytes) -> PResult<(u16, u8)> {
        let l = be_u8.verify(|&l| l >= 131 && l <= 191).parse_next(input)? & 63;
        let c = be_u8.verify(|&c| c > 0).parse_next(input)?;
        Ok((l as u16, c))
    }

    fn word_14bc(input: &mut &Bytes) -> PResult<(u16, u8)> {
        let l = be_u16.verify(|&l| l >= 49216).parse_next(input)? & 16383;
        let c = be_u8.verify(|&c| c > 0).parse_next(input)?;
        Ok((l as u16, c))
    }

    fn word_eol(input: &mut &Bytes) -> PResult<(u16, u8)> {
        literal(0).parse_next(input)?;
        Ok((0, 0))
    }

    alt((word_6b0, word_14b0, word_6bc, word_14bc, word_eol)).parse_next(input)
}

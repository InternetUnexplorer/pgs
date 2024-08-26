use winnow::binary::be_u8;
use winnow::combinator::repeat;
use winnow::{Bytes, PResult, Parser};

pub type Rgba = [u8; 4];

#[derive(Debug, Clone)]
pub struct Pds {
    pub id: u8,
    pub version: u8,
    pub entries: [Rgba; 256],
}

pub fn pds(input: &mut &Bytes) -> PResult<Pds> {
    fn entry(input: &mut &Bytes) -> PResult<(u8, Rgba)> {
        Ok((
            be_u8.parse_next(input)?,
            bt709_to_rgba(
                be_u8.parse_next(input)?,
                be_u8.parse_next(input)?,
                be_u8.parse_next(input)?,
                be_u8.parse_next(input)?,
            ),
        ))
    }

    fn entries(input: &mut &Bytes) -> PResult<[Rgba; 256]> {
        let mut entries = [[0; 4]; 256];
        let entries_vec: Vec<_> = repeat(0..=256, entry).parse_next(input)?;
        entries_vec.into_iter().for_each(|(id, entry)| entries[id as usize] = entry);
        Ok(entries)
    }

    Ok(Pds {
        id: be_u8.parse_next(input)?,
        version: be_u8.parse_next(input)?,
        entries: entries.parse_next(input)?,
    })
}

fn bt709_to_rgba(y: u8, cr: u8, cb: u8, alpha: u8) -> Rgba {
    let y = (y as f32) / 255.0;
    let cr = (cr as f32) / 255.0 - 0.5;
    let cb = (cb as f32) / 255.0 - 0.5;

    let (a, b, c, d, e) = (0.2627, 0.7152, 0.0722, 1.8556, 1.5748);

    let r = y + e * cr;
    let g = y - (a * e / b) * cr - (c * d / b) * cb;
    let b = y + d * cb;

    [
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
        alpha,
    ]
}

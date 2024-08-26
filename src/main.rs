#![allow(dead_code)]

use std::env::args;
use std::{fs, iter};

use lodepng::encode32_file;
use ods::*;
use pds::*;
use segment::*;
use winnow::combinator::repeat;
use winnow::{Bytes, Parser};

mod ods;
mod pds;
mod segment;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();
    let (input, output_dir) = match &args[..] {
        [_, input, output_dir] => (input, output_dir),
        _ => {
            eprintln!("Usage: {} <INPUT> <OUTPUT_DIR>", args[0]);
            std::process::exit(1);
        }
    };

    let input = fs::read(input)?;
    let mut input = Bytes::new(&input);
    fs::create_dir_all(output_dir)?;

    let segments: Vec<Segment> =
        repeat(0.., segment).parse(&mut input).expect("failed to parse segments");
    println!("there are {} segment(s)", segments.len());

    let mut palette = [[0; 4]; 256];
    let mut object_index = 0;
    for segment in segments.iter() {
        match segment.kind {
            0x14 => {
                palette = pds.parse(&mut &segment.body).expect("failed to parse PDS").entries;
            }
            0x15 => {
                let object = ods.parse(&mut &segment.body).expect("failed to parse ODS");
                if !object.is_first_and_last_in_sequence() {
                    panic!("object fragments are not supported yet");
                }
                let (width, height, words) =
                    rle.parse(&mut &object.body).expect("failed to parse RLE");
                let image: Vec<_> = words
                    .into_iter()
                    .flat_map(|word| iter::repeat(palette[word.1 as usize]).take(word.0 as usize))
                    .collect();
                let path = format!("{output_dir}/{object_index}.png");
                eprintln!("writing {path}");
                encode32_file(&path, &image, width as usize, height as usize)?;
                object_index += 1;
            }
            _ => (),
        }
    }
    Ok(())
}

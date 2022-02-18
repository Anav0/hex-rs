use std::{collections::HashMap, env::Args, fs::File, io::Read};

use crate::modes::Modes;
use crossterm::Result;

pub struct Dimensions {
    pub offsets: (u16, u16),
    pub bytes: (u16, u16),
    pub decoded: (u16, u16),
}

impl Dimensions {
    pub fn new(padding: u16, parameters: &Parameters) -> Self {
        let offsets_start = padding;
        let offsets_end = offsets_start + 10;
        let offsets = (offsets_start, offsets_end);

        let bytes_start = offsets_end + 3;
        let bytes_end = (bytes_start + parameters.byte_size * 5) - 1;
        let bytes = (bytes_start, bytes_end);

        let decoded_start = bytes_end + 3;
        let decoded_end = decoded_start + parameters.byte_size;
        let decoded = (decoded_start, decoded_end);

        Self {
            bytes,
            decoded,
            offsets,
        }
    }
}

#[derive(PartialEq)]
pub enum Action {
    Quit,
    DrawBytes,
    DrawHelp,
    Change,
}

#[derive(PartialEq)]
pub enum Direction {
    Left,
    Right,
}

pub enum StatusMode {
    General,
    Keys,
}

pub struct Parameters {
    pub file_path: String,
    pub byte_size: u16,
}

pub struct TermState<'a> {
    pub row: u16,
    pub column: u16,
    pub term_width: u16,
    pub term_height: u16,
    pub padding: u16,
    pub render_from_offset: usize,
    pub status_mode: StatusMode,
    pub dimensions: &'a Dimensions,
    pub prev_mode: Modes,
    //TODO: find a better place for it
    pub bytes: Vec<u8>,
}

impl From<Args> for Parameters {
    fn from(args: Args) -> Self {
        let collected_args: Vec<String> = args.collect();
        let mut byte_size = 16;

        if collected_args.len() >= 3 {
            byte_size = collected_args[2]
                .parse()
                .expect("Second argument must be u16");

            if byte_size <= 0 {
                panic!("Byte size should be greater than 0!")
            }
        }

        Self {
            file_path: collected_args[1].clone(),
            byte_size,
        }
    }
}

pub fn get_bytes(path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path).expect("Failed to open file");
    let file_size = file.metadata()?.len();
    let mut bytes: Vec<u8> = vec![0; file_size as usize];
    file.read(&mut bytes)
        .expect("Failed to read bytes into buffer");

    Ok(bytes)
}

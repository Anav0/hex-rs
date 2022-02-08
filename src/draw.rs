use std::{io::Stdout, io::Write};

use crossterm::{
    cursor, queue,
    style::{self, Color},
    Result,
};

use crate::{keyboard::Keyboard, Parameters, StatusMode, TermState};

pub(crate) fn draw_fixed_ui<W: Write>(
    stdout: &mut W,
    state: &TermState,
    parameters: &Parameters,
    keyboard: &Keyboard,
) -> Result<()> {
    let status = get_status(state, parameters, keyboard);

    queue!(
        stdout,
        style::SetForegroundColor(Color::Yellow),
        cursor::MoveTo(state.padding, 0),
        style::Print("Offset(h)"),
        cursor::MoveTo(state.padding, state.term_height),
        style::Print(status),
        cursor::MoveTo(state.padding + 12, 0),
    )?;

    //Byte columns
    for i in 0..parameters.byte_size {
        queue!(
            stdout,
            cursor::MoveRight(1),
            style::Print(format!("{:#04X}", i))
        )?;
    }
    //Decoded
    queue!(stdout, cursor::MoveRight(3), style::Print("Decoded"))?;
    Ok(())
}

pub(crate) fn draw_bytes(
    stdout: &mut Stdout,
    state: &TermState,
    parameters: &Parameters,
    bytes: &Vec<u8>,
) -> Result<()> {
    //For each byte in file
    let mut byte_x = state.padding + 13;
    let mut byte_y = 1;
    queue!(stdout, style::SetForegroundColor(Color::DarkBlue))?;

    let mut iter = 0;
    let start_from = parameters.byte_size as usize * state.render_from_offset;

    for i in start_from..bytes.len() {
        let byte = bytes[i];
        queue!(
            stdout,
            cursor::MoveTo(byte_x, byte_y),
            style::Print(format!("{:#04X}", byte))
        )?;

        byte_x += 5;
        iter += 1;

        //Overflow on x axis
        if iter >= parameters.byte_size || i == bytes.len() - 1 {
            queue!(stdout, cursor::MoveTo(97, byte_y))?;
            for j in i + 1 - iter as usize..=i {
                let decoded = get_symbol(bytes[j]);
                queue!(stdout, cursor::MoveRight(0), style::Print(decoded))?;
            }
            iter = 0;
            byte_x = state.padding + 13;
            byte_y += 1;
        }

        //Overflow on y axis (columns)
        if byte_y >= state.term_height {
            break;
        }
    }
    Ok(())
}

pub(crate) fn draw_offsets(
    stdout: &mut Stdout,
    state: &TermState,
    parameters: &Parameters,
    offsets: u16,
) -> Result<()> {
    let mut iter = 0;
    for i in state.render_from_offset as u16..offsets + 1 {
        if iter >= state.term_height - 1 {
            break;
        }
        queue!(
            stdout,
            style::SetForegroundColor(Color::Yellow),
            cursor::MoveTo(state.padding, iter + 1 as u16),
            style::Print(format!("{:#010x}", i * parameters.byte_size))
        )?;
        iter += 1;
    }
    Ok(())
}

fn get_status(state: &TermState, parameters: &Parameters, keyboard: &Keyboard) -> String {
    match state.status_mode {
        StatusMode::General => format!(
            "Hex Editor ({}x{}) - {}:{}, file: {}",
            state.term_width, state.term_height, state.column, state.row, &parameters.file_path
        ),
        StatusMode::Keys => keyboard.help(),
    }
}

fn get_symbol(byte: u8) -> char {
    if byte.is_ascii_whitespace() {
        return ' ';
    }

    if !byte.is_ascii() || byte.is_ascii_control() {
        return '.';
    }

    char::from(byte)
}

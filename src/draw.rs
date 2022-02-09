use std::{io::Stdout, io::Write, ops::Range};

use crossterm::{
    cursor, queue,
    style::{self, Color, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
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
        style::SetBackgroundColor(Color::Reset),
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
pub(crate) fn draw_help(stdout: &mut Stdout, keyboard: &Keyboard, state: &TermState) -> Result<()> {
    let help_text = keyboard.help("\n");
    let help_items = help_text.lines();

    let mut i = state.padding;
    queue!(stdout, terminal::Clear(ClearType::All))?;

    for line in help_items {
        let splited: Vec<&str> = line.split(": ").collect();
        let move_by = (20 - splited[0].len()) as u16;
        queue!(
            stdout,
            cursor::MoveTo(state.padding, i),
            style::Print(splited[0]),
            cursor::MoveRight(move_by),
            style::Print(splited[1])
        )?;
        i += 1;
    }

    stdout.flush()?;

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

        if byte_y == state.row && byte_x == state.column {
            queue!(
                stdout,
                cursor::MoveTo(byte_x, byte_y),
                SetForegroundColor(Color::DarkBlue),
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(byte_x, byte_y),
                SetBackgroundColor(Color::Reset),
                SetForegroundColor(Color::DarkGrey),
            )?;
        }

        queue!(stdout, style::Print(format!("{:#04X}", byte)))?;

        byte_x += 5;
        iter += 1;

        //Overflow on x axis, time to print decoded chars
        if iter >= parameters.byte_size || i == bytes.len() - 1 {
            let start = i + 1 - iter as usize;
            let end = i;
            let range = Range { start, end };

            let starting_pos = (state.dimensions.decoded.0, byte_y);

            draw_chars(stdout, state, starting_pos, range, bytes, byte_y)?;

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

fn draw_chars<W: Write>(
    stdout: &mut W,
    state: &TermState,
    starting_pos: (u16, u16),
    range: Range<usize>,
    bytes: &Vec<u8>,
    byte_y: u16,
) -> Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(starting_pos.0, starting_pos.1),
        SetForegroundColor(Color::DarkGrey)
    )?;
    let mut char_pos = 0;
    for i in range {
        let required_y = state.dimensions.bytes.0 + 5 * char_pos;
        let decoded = get_symbol(bytes[i]);

        let mut fg = Color::DarkGrey;
        let mut bg = Color::Reset;

        if byte_y == state.row && required_y == state.column {
            if decoded == ' ' {
                bg = Color::DarkBlue;
            }
            fg = Color::DarkBlue;
        }
        queue!(
            stdout,
            cursor::MoveRight(0),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            style::Print(decoded)
        )?;
        char_pos += 1;
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
            style::SetBackgroundColor(Color::Reset),
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
        StatusMode::Keys => keyboard.help(", "),
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

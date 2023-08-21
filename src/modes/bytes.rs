use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Stdout, Write};
use std::ops::Range;

use crossterm::event::{self, KeyEvent, MouseEvent};
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::ClearType;
use crossterm::{cursor, queue, style, terminal};

use crate::StatusMode;
use crate::{keyboard::Keyboard, Parameters, TermState};

use super::{Mode, Modes};
enum BytesScreens {
    Bytes,
    TooSmall,
}
pub struct BytesMode<'a> {
    keyboard: &'a Keyboard<'a>,
    parameters: &'a Parameters,
    offsets: u16,
    minimal_width: u16,
    to_draw: BytesScreens,
}
impl<'a> BytesMode<'a> {
    fn draw_too_small(&self, stdout: &mut Stdout) -> Result<()> {
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(1, 1),
            style::Print(format!(
                "Windows too small to display {} bytes in one row",
                self.parameters.byte_size
            ))
        )?;
        Ok(())
    }
    pub fn new(
        keyboard: &'a Keyboard,
        parameters: &'a Parameters,
        file_size: usize,
    ) -> Result<BytesMode<'a>> {
        let offsets = file_size as u16 / parameters.byte_size;
        let minimal_width = ((parameters.byte_size + 1) * 5) + 16;

        let mode = BytesMode {
            keyboard,
            parameters,
            offsets,
            minimal_width,
            to_draw: BytesScreens::Bytes,
        };

        Ok(mode)
    }
}
impl<'a> Mode for BytesMode<'a> {
    fn handle_input(
        &mut self,
        event: &KeyEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes> {
        match self.keyboard.get(&event) {
            Some(action) => Ok(action(state, parameters)),
            None => Ok(Modes::Bytes),
        }
    }

    fn handle_mouse(
        &mut self,
        event: &MouseEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes> {
        match event.kind {
            event::MouseEventKind::ScrollDown => state.render_from_offset += 1,
            event::MouseEventKind::ScrollUp => state.render_from_offset -= 1,
            event::MouseEventKind::Up(btn) => match btn {
                event::MouseButton::Left => {
                    state.column = event.column;
                    state.row = event.row;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(Modes::Bytes)
    }

    fn handle_resize(
        &mut self,
        stdout: &mut Stdout,
        width: u16,
        height: u16,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes> {
        if width < self.minimal_width {
            self.to_draw = BytesScreens::TooSmall;

            //Check if cursor is not left behind
            if state.column > state.term_width {
                state.column = state.term_width
            }
            if state.row > state.term_height {
                state.row = state.term_height
            }

            return Ok(Modes::Bytes);
        }

        self.to_draw = BytesScreens::Bytes;
        state.term_width = width;
        state.term_height = height;

        Ok(Modes::Bytes)
    }

    fn draw(&self, stdout: &mut Stdout, state: &TermState) -> Result<()> {
        match self.to_draw {
            BytesScreens::Bytes => {
                queue!(stdout, terminal::Clear(ClearType::All))?;

                draw_fixed_ui(stdout, &state, &self.parameters, &self.keyboard)?;
                draw_offsets(stdout, &state, &self.parameters, self.offsets)?;
                draw_bytes(stdout, &state, &self.parameters, &state.bytes)?;

                queue!(stdout, cursor::MoveTo(state.column, state.row))?;
            }
            BytesScreens::TooSmall => {
                self.draw_too_small(stdout)?;
            }
        }

        Ok(())
    }
}

fn draw_bytes(
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

    let mut fg_info: HashMap<usize, Color> = HashMap::new();
    let mut bg_info: HashMap<usize, Color> = HashMap::new();

    for i in start_from..bytes.len() {
        let byte = bytes[i];

        queue!(
            stdout,
            SetBackgroundColor(Color::Reset),
            cursor::MoveTo(byte_x, byte_y),
        )?;

        let mut fg = Color::DarkGrey;
        let mut bg = Color::Reset;

        // Check if byte is in one of found sequences
        //@Improvement: change to something nicer
        for range in &state.found_sequences {
            if range.contains(&i) {
                fg = Color::White;
                break;
            }
        }

        //@Improvement: change to something nicer
        if byte_y == state.row && byte_x == state.column {
            fg = Color::DarkBlue;
        } else if state.bytes_removed.contains(&i) {
            fg = Color::Red;
        } else if state.bytes_changed.contains(&i) {
            fg = Color::DarkBlue;
        }

        fg_info.insert(i, fg);
        bg_info.insert(i, bg);

        queue!(
            stdout,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            style::Print(format!("{:#04X}", byte))
        )?;

        byte_x += 5;
        iter += 1;

        //Overflow on x axis, time to print decoded chars
        if iter >= parameters.byte_size || i == bytes.len() - 1 {
            let start = i + 1 - iter as usize;
            let end = i + 1;
            let range = Range { start, end };

            let starting_pos = (state.dimensions.decoded.0, byte_y);

            draw_chars(stdout, starting_pos, range, bytes, &fg_info, &bg_info)?;

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
    starting_pos: (u16, u16),
    range: Range<usize>,
    bytes: &Vec<u8>,
    fg_info: &HashMap<usize, Color>,
    bg_info: &HashMap<usize, Color>,
) -> Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(starting_pos.0, starting_pos.1),
        SetForegroundColor(Color::DarkGrey)
    )?;
    for i in range {
        let decoded = get_symbol(bytes[i]);

        let fg = fg_info.get(&i).unwrap();
        let mut bg = bg_info.get(&i).unwrap();

        if decoded == ' ' {
            bg = fg;
        }

        queue!(
            stdout,
            cursor::MoveRight(0),
            SetForegroundColor(*fg),
            SetBackgroundColor(*bg),
            style::Print(decoded)
        )?;
    }

    Ok(())
}

fn draw_offsets(
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
        StatusMode::General => {
            let mut status = format!(
                "Hex Editor ({}x{}) - {}:{}, file: {}",
                state.term_width, state.term_height, state.column, state.row, &parameters.file_path
            );

            if state.bytes_changed.len() > 0 {
                let bytes_info = format!(", Bytes changes: {}", state.bytes_changed.len());
                status.push_str(&bytes_info);
            }

            status
        }
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

fn draw_fixed_ui<W: Write>(
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

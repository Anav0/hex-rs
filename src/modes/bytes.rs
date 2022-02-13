use std::fs::File;
use std::io::{Read, Stdout, Write};
use std::ops::Range;

use crossterm::event::{self, KeyEvent, MouseEvent};
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::ClearType;
use crossterm::{cursor, queue, style, terminal, Result};

use crate::StatusMode;
use crate::{keyboard::Keyboard, Parameters, TermState};

use super::{Mode, Modes};

pub struct BytesMode<'a> {
    keyboard: &'a Keyboard<'a>,
    state: &'a mut TermState<'a>,
    parameters: &'a Parameters,
    bytes: Vec<u8>,
    offsets: u16,
    quit: bool,
    minimal_width: u16,
}
impl<'a> BytesMode<'a> {
    pub fn new(
        keyboard: &'a Keyboard,
        state: &'a mut TermState<'a>,
        parameters: &'a Parameters,
    ) -> Result<BytesMode<'a>> {
        let bytes = get_bytes(&parameters.file_path)?;
        let file_size = bytes.len();
        let offsets = file_size as u16 / parameters.byte_size;
        let minimal_width = ((parameters.byte_size + 1) * 5) + 16;

        let mode = BytesMode {
            keyboard,
            state,
            parameters,
            bytes,
            offsets,
            minimal_width,
            quit: false,
        };

        Ok(mode)
    }
}
impl<'a> Mode for BytesMode<'a> {
    fn handle_input(&mut self, event: &KeyEvent) -> Result<Modes> {
        let action = self
            .keyboard
            .get(&event.code)
            .expect(&format!("Failed to handle key: '{:?}'", event.code));
        let action = action(self.state);

        match action {
            crate::Action::Quit => self.quit = true,
            crate::Action::DrawHelp => return Ok(Modes::Help),
            _ => {}
        }

        Ok(Modes::Bytes)
    }

    fn handle_mouse(&mut self, event: &MouseEvent) -> Result<Modes> {
        match event.kind {
            event::MouseEventKind::ScrollDown => self.state.render_from_offset += 1,
            event::MouseEventKind::ScrollUp => self.state.render_from_offset -= 1,
            event::MouseEventKind::Up(btn) => match btn {
                event::MouseButton::Left => {
                    self.state.column = event.column;
                    self.state.row = event.row;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(Modes::Bytes)
    }

    fn handle_resize(&mut self, stdout: &mut Stdout, width: u16, height: u16) -> Result<Modes> {
        if width < self.minimal_width {
            queue!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(1, 1),
                style::Print(format!(
                    "Windows too small to display {} bytes in one row",
                    self.parameters.byte_size
                ))
            )?;
            stdout.flush()?;

            //Check if cursor is not left behind
            if self.state.column > self.state.term_width {
                self.state.column = self.state.term_width
            }
            if self.state.row > self.state.term_height {
                self.state.row = self.state.term_height
            }

            return Ok(Modes::Bytes);
        }

        self.state.term_width = width;
        self.state.term_height = height;

        Ok(Modes::Bytes)
    }

    fn draw(&self, stdout: &mut Stdout) -> Result<()> {
        queue!(stdout, terminal::Clear(ClearType::All))?;

        draw_fixed_ui(stdout, &self.state, &self.parameters, &self.keyboard)?;
        draw_offsets(stdout, &self.state, &self.parameters, self.offsets)?;
        draw_bytes(stdout, &self.state, &self.parameters, &self.bytes)?;

        queue!(stdout, cursor::MoveTo(self.state.column, self.state.row))?;

        Ok(())
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}
fn get_bytes(path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path).expect("Failed to open file");
    let file_size = file.metadata()?.len();
    let mut bytes: Vec<u8> = vec![0; file_size as usize];
    file.read(&mut bytes)
        .expect("Failed to read bytes into buffer");

    Ok(bytes)
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
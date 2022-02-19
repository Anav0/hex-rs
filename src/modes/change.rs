use crossterm::{
    cursor,
    event::KeyCode,
    queue, style,
    terminal::{self, ClearType},
};

use crate::misc::{get_byte_at_cursor, Parameters, TermState};

use super::{Mode, Modes};

pub struct ChangeMode<'a> {
    pub input: String,
    parameters: &'a Parameters,
}

impl<'a> ChangeMode<'a> {
    pub fn new(parameters: &'a Parameters) -> Self {
        Self {
            input: String::from(""),
            parameters,
        }
    }
}

impl<'a> Mode for ChangeMode<'a> {
    fn handle_input(
        &mut self,
        event: &crossterm::event::KeyEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> crossterm::Result<super::Modes> {
        let end_mode = match event.code {
            KeyCode::Char(char) => {
                if char == 'q' {
                    self.input.clear();
                    return Ok(Modes::Bytes);
                }
                if !char.is_ascii_hexdigit() || self.input.len() >= 2 {
                    return Ok(Modes::Change);
                }
                self.input.push(char.to_ascii_uppercase());
                Modes::Change
            }
            KeyCode::Backspace => {
                self.input.pop();
                Modes::Change
            }
            KeyCode::Enter => {
                if self.input.len() != 2 {
                    return Ok(Modes::Change);
                }

                let byte =
                    u8::from_str_radix(&self.input, 16).expect("Failed to convert input to byte");

                let byte_index = get_byte_at_cursor(state, self.parameters);

                state.bytes[byte_index] = byte;

                if !state.bytes_changed.contains(&byte_index) {
                    state.bytes_changed.insert(byte_index);
                }

                self.input.clear();

                Modes::Bytes
            }
            _ => Modes::Change,
        };

        Ok(end_mode)
    }

    fn handle_mouse(
        &mut self,
        event: &crossterm::event::MouseEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> crossterm::Result<super::Modes> {
        Ok(Modes::Change)
    }

    fn handle_resize(
        &mut self,
        stdout: &mut std::io::Stdout,
        width: u16,
        height: u16,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> crossterm::Result<super::Modes> {
        Ok(Modes::Change)
    }

    fn should_quit(&self) -> bool {
        false
    }

    fn draw(&self, stdout: &mut std::io::Stdout, state: &TermState) -> crossterm::Result<()> {
        for i in 1..4 {
            queue!(
                stdout,
                cursor::MoveTo(state.column + i, state.row),
                style::Print(" ")
            )?;
        }
        queue!(
            stdout,
            cursor::MoveTo(state.column, state.row),
            style::Print(format!("0x{}", &self.input))
        )?;
        Ok(())
    }
}

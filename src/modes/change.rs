use crossterm::{
    cursor,
    event::KeyCode,
    queue, style,
    terminal::{self, ClearType},
};

use crate::misc::{Parameters, TermState};

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
                let byte =
                    u8::from_str_radix(&self.input, 16).expect("Failed to convert input to byte");

                let bytes_section_column = state.dimensions.bytes.0;

                // @Improvement: Move "5" (hex value width + space) to separate variable
                let actual_row = state.row - 1;
                let actual_column = (state.column - bytes_section_column) / 5;

                let byte_index = (actual_row * self.parameters.byte_size + actual_column) as usize;

                state.bytes[byte_index] = byte;

                if !state.bytes_changed.contains(&(state.column, state.row)) {
                    state.bytes_changed.insert((state.column, state.row));
                }

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
    ) -> crossterm::Result<super::Modes> {
        Ok(Modes::Change)
    }

    fn handle_resize(
        &mut self,
        stdout: &mut std::io::Stdout,
        width: u16,
        height: u16,
        state: &mut TermState,
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

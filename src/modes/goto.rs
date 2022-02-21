use crossterm::{cursor, event::KeyCode, queue, style, Result};

use super::{Mode, Modes};

pub struct GoToMode {
    input: String,
    cursor: usize,
}
impl GoToMode {
    pub fn new() -> Self {
        Self {
            input: String::from(""),
            cursor: 0,
        }
    }
}

impl Mode for GoToMode {
    fn handle_input(
        &mut self,
        event: &crossterm::event::KeyEvent,
        state: &mut crate::misc::TermState,
        parameters: &crate::misc::Parameters,
    ) -> Result<super::Modes> {
        let end_mode = match event.code {
            KeyCode::Char(char) => {
                if char == 'q' {
                    self.input.clear();
                    return Ok(Modes::Bytes);
                }
                if !char.is_ascii_hexdigit() || self.input.len() >= 2 {
                    return Ok(Modes::GoTo);
                }
                self.input.push(char.to_ascii_uppercase());
                Modes::GoTo
            }
            KeyCode::Backspace => Modes::GoTo,
            KeyCode::Enter => Modes::Bytes,
            _ => Modes::GoTo,
        };

        Ok(end_mode)
    }

    fn handle_mouse(
        &mut self,
        event: &crossterm::event::MouseEvent,
        state: &mut crate::misc::TermState,
        parameters: &crate::misc::Parameters,
    ) -> Result<super::Modes> {
        Ok(Modes::GoTo)
    }

    fn handle_resize(
        &mut self,
        stdout: &mut std::io::Stdout,
        width: u16,
        height: u16,
        state: &mut crate::misc::TermState,
        parameters: &crate::misc::Parameters,
    ) -> Result<super::Modes> {
        Ok(Modes::GoTo)
    }

    fn draw(&self, stdout: &mut std::io::Stdout, state: &crate::misc::TermState) -> Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(1, state.term_height),
            style::Print(format!("Go to offset: {}", self.input))
        )?;

        Ok(())
    }
}

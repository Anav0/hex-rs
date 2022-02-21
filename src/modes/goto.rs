use crossterm::{
    cursor::{self, CursorShape},
    event::KeyCode,
    queue,
    style::{self, Color},
    terminal::{self, ClearType},
    Result,
};

use super::{Mode, Modes};

pub struct GoToMode {
    input: String,
    cursor: usize,
    draw_error: bool,
}
impl GoToMode {
    pub fn new() -> Self {
        Self {
            input: String::from("00000000"),
            cursor: 24, //@Improve: base this value on msg length
            draw_error: false,
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
        if self.draw_error {
            self.draw_error = false;
        }
        let end_mode = match event.code {
            KeyCode::Right => {
                //@Improve: base this value on msg length
                if self.cursor < 24 {
                    self.cursor += 1;
                }
                Modes::GoTo
            }
            KeyCode::Left => {
                if self.cursor > 17 {
                    //@Improve: base this value on msg length
                    self.cursor -= 1;
                }
                Modes::GoTo
            }
            KeyCode::Char(char) => {
                if char == 'q' {
                    return Ok(Modes::Bytes);
                }
                if !char.is_ascii_hexdigit() {
                    return Ok(Modes::GoTo);
                }

                self.input.remove(self.cursor - 17);
                self.input
                    .insert(self.cursor - 17, char.to_ascii_uppercase());

                Modes::GoTo
            }
            KeyCode::Enter => {
                let total_number_of_offsets = state.bytes.len() / parameters.byte_size as usize;
                let number = usize::from_str_radix(&self.input, 16)
                    .expect("Failed to parse offset as usize");

                let goto = number / parameters.byte_size as usize;

                if goto <= total_number_of_offsets {
                    state.render_from_offset = goto;
                } else {
                    self.draw_error = true;
                    return Ok(Modes::GoTo);
                }

                Modes::Bytes
            }
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
        let msg = format!("Go to offset: 0x{}", self.input);

        if self.draw_error {
            queue!(
                stdout,
                cursor::MoveTo(1, state.term_height),
                terminal::Clear(ClearType::FromCursorDown),
                style::SetForegroundColor(Color::Red),
                style::Print("Offset exceeds total number of offsets!"),
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(1, state.term_height),
                terminal::Clear(ClearType::FromCursorDown),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print(&msg),
                cursor::SetCursorShape(CursorShape::Block),
                cursor::MoveTo(self.cursor as u16, state.term_height),
            )?;
        }

        Ok(())
    }
}

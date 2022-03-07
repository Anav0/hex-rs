use crossterm::{
    cursor::{self, CursorShape},
    event::KeyCode,
    queue,
    style::{self, Color},
    terminal::{self, ClearType},
    Result,
};

use crate::string::naive_search;

use super::{Mode, Modes};

pub struct SearchMode {
    input: String,
    cursor: usize,
    draw_error: bool,
}

impl SearchMode {
    pub fn new() -> Self {
        Self {
            input: String::from(""),
            cursor: 13, //@Improve: base this value on msg length
            draw_error: false,
        }
    }
}

impl Mode for SearchMode {
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
                self.cursor += 1;
                Modes::Search
            }
            KeyCode::Left => {
                if self.cursor > 13 {
                    //@Improve: base this value on msg length
                    self.cursor -= 1;
                }
                Modes::Search
            }
            KeyCode::Backspace => {
                if self.input.len() > 0 {
                    self.input.remove(self.cursor - 14);
                    self.cursor -= 1;
                }
                Modes::Search
            }
            KeyCode::Char(char) => {
                if char == 'q' {
                    return Ok(Modes::Bytes);
                }
                if !char.is_ascii_hexdigit() {
                    return Ok(Modes::Search);
                }

                self.input.push(char);
                self.cursor += 1;

                Modes::Search
            }
            KeyCode::Enter => {
                if self.input.len() == 0 {
                    return Ok(Modes::Search);
                }

                state.found_sequences.clear();

                let mut bytes = vec![];

                let mut counter = 0;
                loop {
                    if counter >= self.input.len() {
                        break;
                    }

                    let offset = match counter + 1 < self.input.len() {
                        true => 2,
                        false => 1,
                    };

                    let slice = self.input.get(counter..counter + offset).unwrap();
                    bytes.push(u8::from_str_radix(slice, 16).unwrap());

                    counter += offset;
                }

                state.found_sequences = naive_search(&bytes, &state.bytes);

                Modes::Bytes
            }
            _ => Modes::Search,
        };

        Ok(end_mode)
    }

    fn handle_mouse(
        &mut self,
        event: &crossterm::event::MouseEvent,
        state: &mut crate::misc::TermState,
        parameters: &crate::misc::Parameters,
    ) -> Result<super::Modes> {
        Ok(Modes::Search)
    }

    fn handle_resize(
        &mut self,
        stdout: &mut std::io::Stdout,
        width: u16,
        height: u16,
        state: &mut crate::misc::TermState,
        parameters: &crate::misc::Parameters,
    ) -> Result<super::Modes> {
        Ok(Modes::Search)
    }

    fn draw(&self, stdout: &mut std::io::Stdout, state: &crate::misc::TermState) -> Result<()> {
        let msg = format!("Search for: {}", self.input);

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

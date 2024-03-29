use std::io::{Result, Stdout};

use crossterm::cursor;
use crossterm::event::KeyEvent;
use crossterm::event::MouseEvent;
use crossterm::queue;
use crossterm::style;
use crossterm::terminal;
use crossterm::terminal::ClearType;

use crate::keyboard::Keyboard;
use crate::misc::Parameters;
use crate::misc::TermState;

use super::{Mode, Modes};

pub struct HelpMode<'a> {
    padding: u16,
    keyboard: &'a Keyboard<'a>,
}
impl<'a> HelpMode<'a> {
    pub fn new(padding: u16, keyboard: &'a Keyboard) -> Self {
        Self { padding, keyboard }
    }
}

impl<'a> Mode for HelpMode<'a> {
    fn handle_input(
        &mut self,
        event: &KeyEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes> {
        Ok(Modes::Bytes)
    }

    fn handle_mouse(
        &mut self,
        event: &MouseEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes> {
        Ok(Modes::Help)
    }

    fn handle_resize(
        &mut self,
        stdout: &mut Stdout,
        width: u16,
        height: u16,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes> {
        Ok(Modes::Help)
    }

    fn draw(&self, stdout: &mut Stdout, state: &TermState) -> Result<()> {
        let help_text = self.keyboard.help("\n");
        let help_items = help_text.lines();

        let mut i = self.padding;
        queue!(stdout, terminal::Clear(ClearType::All))?;

        for line in help_items {
            let splited: Vec<&str> = line.split(": ").collect();
            let move_by = (20 - splited[0].len()) as u16;
            queue!(
                stdout,
                cursor::MoveTo(self.padding, i),
                style::Print(splited[0]),
                cursor::MoveRight(move_by),
                style::Print(splited[1])
            )?;
            i += 1;
        }

        Ok(())
    }
}

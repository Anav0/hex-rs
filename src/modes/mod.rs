use crossterm::event::{KeyEvent, MouseEvent};
use crossterm::Result;
use std::io::Stdout;

mod bytes;
mod change;
mod goto;
mod help;

pub use bytes::BytesMode;
pub use change::ChangeMode;
pub use goto::GoToMode;
pub use help::HelpMode;

use crate::misc::{Parameters, TermState};

#[derive(PartialEq)]
pub enum Modes {
    Bytes,
    Help,
    Change,
    GoTo,
    Quit,
}

pub trait Mode {
    fn handle_input(
        &mut self,
        event: &KeyEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes>;
    fn handle_mouse(
        &mut self,
        event: &MouseEvent,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes>;
    fn handle_resize(
        &mut self,
        stdout: &mut Stdout,
        width: u16,
        height: u16,
        state: &mut TermState,
        parameters: &Parameters,
    ) -> Result<Modes>;
    fn draw(&self, stdout: &mut Stdout, state: &TermState) -> Result<()>;
}

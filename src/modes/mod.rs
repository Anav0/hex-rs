use crossterm::event::{KeyEvent, MouseEvent};
use crossterm::Result;
use std::io::Stdout;

mod bytes;
mod help;

pub use bytes::BytesMode;
pub use help::HelpMode;

#[derive(PartialEq)]
pub enum Modes {
    Bytes,
    Help,
}

pub trait Mode {
    fn handle_input(&mut self, event: &KeyEvent) -> Result<Modes>;
    fn handle_mouse(&mut self, event: &MouseEvent) -> Result<Modes>;
    fn handle_resize(&mut self, stdout: &mut Stdout, width: u16, height: u16) -> Result<Modes>;
    fn should_quit(&self) -> bool;
    fn draw(&self, stdout: &mut Stdout) -> Result<()>;
}

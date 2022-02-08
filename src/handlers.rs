use std::io::{Stdout, Write};

use crossterm::{
    cursor,
    event::{self, KeyEvent},
    queue, style,
    terminal::{self, ClearType},
    Result,
};

use crate::{keyboard::Keyboard, Action, Parameters, TermState};

pub(crate) fn handle_resize(
    stdout: &mut Stdout,
    state: &mut TermState,
    width: u16,
    height: u16,
    minimal_width: u16,
    parameters: &Parameters,
) -> Result<Action> {
    if width < minimal_width {
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(1, 1),
            style::Print(format!(
                "Windows too small to display {} bytes in one row",
                parameters.byte_size
            ))
        )?;
        stdout.flush()?;

        //Check if cursor is not left behind
        if state.column > state.term_width {
            state.column = state.term_width
        }
        if state.row > state.term_height {
            state.row = state.term_height
        }

        return Ok(Action::SkipDrawing);
    }

    state.term_width = width;
    state.term_height = height;

    Ok(Action::DrawBytes)
}

pub(crate) fn handle_mouse(state: &mut TermState, event: event::MouseEvent) -> Action {
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
    Action::DrawBytes
}

pub(crate) fn handle_input(state: &mut TermState, event: KeyEvent, keyboard: &Keyboard) -> Action {
    match keyboard.get(&event.code) {
        Some(action) => action(state),
        None => Action::DrawBytes,
    }
}

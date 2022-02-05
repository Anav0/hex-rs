use std::{
    env::{self, Args},
    io::{self, stdout, Stdout, Write},
    os::windows::thread,
    ptr::NonNull,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color},
    terminal::ClearType,
};
use crossterm::{terminal, Result};

struct Parameters {
    file_path: String,
}

struct TermState {
    pub row: u16,
    pub column: u16,
    pub term_width: u16,
    pub term_height: u16,
    pub padding: u16,
}

impl From<Args> for Parameters {
    fn from(args: Args) -> Self {
        let collected_args: Vec<String> = args.collect();
        Self {
            file_path: collected_args[1].clone(),
        }
    }
}

fn main() -> Result<()> {
    let args = env::args();
    let parameters = Parameters::from(args);

    let mut stdout = stdout();

    //Enter terminal application mode
    execute!(&mut stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let size = terminal::size()?;

    let mut state = TermState {
        row: 1,
        column: 1,
        term_height: size.1,
        term_width: size.0,
        padding: 2,
    };

    loop {
        if poll(Duration::from_millis(100))? {
            let code = match read()? {
                Event::Key(event) => handle_input(&mut state, event),
                Event::Resize(width, height) => {
                    state.term_width = width;
                    state.term_height = height;
                    0
                }
                _ => 0,
            };

            if code == 1 {
                break;
            }

            let status = format!(
                "Hex Editor ({}x{}) - {}:{}",
                state.term_width, state.term_height, state.column, state.row
            );

            if state.column > state.term_width {
                state.column = state.term_width
            }
            if state.row > state.term_height {
                state.row = state.term_height
            }

            queue!(
                &mut stdout,
                terminal::Clear(ClearType::All),
                style::SetForegroundColor(Color::Yellow),
                cursor::MoveTo(state.padding, state.term_height),
                style::Print(status),
                cursor::MoveTo(state.column, state.row),
                style::SetForegroundColor(Color::DarkBlue),
            )?;

            stdout.flush()?;
        }
    }

    //Bring terminal back to normal
    execute!(
        &mut stdout,
        terminal::Clear(ClearType::All),
        terminal::LeaveAlternateScreen,
        cursor::Show,
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn handle_input(state: &mut TermState, event: KeyEvent) -> u8 {
    match event.code {
        KeyCode::Up => {
            if state.row != 0 {
                state.row -= 1;
            }
        }
        KeyCode::Left => {
            if state.column != 0 {
                state.column -= 1;
            }
        }
        KeyCode::Down => {
            if state.row != state.term_height {
                state.row += 1;
            }
        }
        KeyCode::Right => {
            if state.column != state.term_width {
                state.column += 1;
            }
        }
        KeyCode::PageUp => state.padding += 1,
        KeyCode::PageDown => {
            if state.padding != 0 {
                state.padding -= 1
            }
        }
        KeyCode::Char(char) => match char {
            'q' => return 1,
            _ => return 0,
        },
        _ => {}
    }
    0
}

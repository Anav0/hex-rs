use std::{
    collections::HashMap,
    env::{self},
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, read, Event},
    execute,
    terminal::ClearType,
};
use crossterm::{terminal, Result};
use keyboard::Keyboard;
use misc::{get_bytes, Dimensions, Parameters, StatusMode, TermState};
use modes::{BytesMode, ChangeMode, HelpMode, Mode, Modes};

mod actions;
mod keyboard;
mod misc;
mod modes;

fn main() -> Result<()> {
    let args = env::args();
    let parameters = Parameters::from(args);

    let mut stdout = stdout();

    //Enter terminal application mode
    execute!(&mut stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let size = terminal::size()?;
    let padding = 2;
    let dimensions = Dimensions::new(padding, &parameters);
    let keyboard = Keyboard::new();

    let bytes = get_bytes(&parameters.file_path)?;
    let file_size = bytes.len();

    let mut state = TermState {
        row: 1,
        column: dimensions.bytes.0,
        term_height: size.1,
        term_width: size.0,
        padding,
        render_from_offset: 0,
        status_mode: StatusMode::General,
        dimensions: &dimensions,
        prev_mode: Modes::Bytes,
        bytes,
    };

    // Modes
    let mut bytes_mode = BytesMode::new(&keyboard, &parameters, file_size)?;
    let mut help_mode = HelpMode::new(padding, &keyboard);
    let mut change_mode = ChangeMode::new(&parameters);
    let modes: [&mut dyn Mode; 3] = [&mut bytes_mode, &mut help_mode, &mut change_mode];

    let mut index = 0;

    loop {
        if poll(Duration::from_millis(16))? {
            let new_mode = match read()? {
                Event::Key(event) => modes[index].handle_input(&event, &mut state)?,
                Event::Mouse(event) => modes[index].handle_mouse(&event, &mut state)?,
                Event::Resize(width, height) => {
                    modes[index].handle_resize(&mut stdout, width, height, &mut state)?
                }
            };

            if modes[index].should_quit() {
                break;
            }

            let new_index = match new_mode {
                Modes::Bytes => 0,
                Modes::Help => 1,
                Modes::Change => 2,
            };

            if new_index != index {
                index = new_index;
            }

            modes[index].draw(&mut stdout, &state)?;

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

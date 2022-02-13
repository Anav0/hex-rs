use std::{
    env::{self, Args},
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
use misc::{Dimensions, Parameters, StatusMode, TermState};
use modes::{BytesMode, HelpMode, Mode, Modes};

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
    };

    // Modes
    let mut help_mode = HelpMode::new(state.padding, &keyboard);
    let mut bytes_mode = BytesMode::new(&keyboard, &mut state, &parameters)?;
    let modes: [&mut dyn Mode; 2] = [&mut bytes_mode, &mut help_mode];

    let mut index = 0;

    loop {
        if poll(Duration::from_millis(16))? {
            let new_mode = match read()? {
                Event::Key(event) => modes[index].handle_input(&event)?,
                Event::Mouse(event) => modes[index].handle_mouse(&event)?,
                Event::Resize(width, height) => {
                    modes[index].handle_resize(&mut stdout, width, height)?
                }
            };

            if modes[index].should_quit() {
                break;
            }

            let new_index = match new_mode {
                Modes::Bytes => 0,
                Modes::Help => 1,
            };

            if new_index != index {
                index = new_index;
            }

            modes[index].draw(&mut stdout)?;

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

use std::{
    collections::{HashMap, HashSet},
    env::{self},
    fs::{File, OpenOptions},
    io::{stdout, Read, Write},
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
use modes::{BytesMode, ChangeMode, GoToMode, HelpMode, Mode, Modes, SearchMode};

mod actions;
mod keyboard;
mod misc;
mod modes;
mod string;

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

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .open(parameters.file_path.clone())
        .expect("Failed to open file");

    let file_size = file.metadata()?.len();
    let mut bytes: Vec<u8> = vec![0; file_size as usize];
    file.read(&mut bytes)
        .expect("Failed to read bytes into buffer");

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
        bytes_changed: HashSet::new(),
        bytes_removed: HashSet::new(),
        bytes,
        found_sequences: HashSet::new(),
        file_path: &parameters.file_path,
    };

    // Modes
    let mut bytes_mode = BytesMode::new(&keyboard, &parameters, file_size as usize)?;
    let mut help_mode = HelpMode::new(padding, &keyboard);
    let mut change_mode = ChangeMode::new(&parameters);
    let mut search_mode = SearchMode::new();
    let mut goto_mode = GoToMode::new();
    let modes: [&mut dyn Mode; 5] = [
        &mut bytes_mode,
        &mut help_mode,
        &mut change_mode,
        &mut goto_mode,
        &mut search_mode,
    ];

    let mut index = 0;

    loop {
        if poll(Duration::from_millis(16))? {
            let new_mode = match read()? {
                Event::Key(event) => modes[index].handle_input(&event, &mut state, &parameters)?,
                Event::Mouse(event) => {
                    modes[index].handle_mouse(&event, &mut state, &parameters)?
                }
                Event::Resize(width, height) => modes[index].handle_resize(
                    &mut stdout,
                    width,
                    height,
                    &mut state,
                    &parameters,
                )?,
            };

            let new_index = match new_mode {
                Modes::Bytes => 0,
                Modes::Help => 1,
                Modes::Change => 2,
                Modes::GoTo => 3,
                Modes::Search => 4,
                Modes::Quit => break,
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

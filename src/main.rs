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
        bytes,
        file_path: &parameters.file_path,
    };

    // Modes
    let mut bytes_mode = BytesMode::new(&keyboard, &parameters, file_size as usize)?;
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

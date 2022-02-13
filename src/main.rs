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
use modes::{BytesMode, HelpMode, Mode, Modes};

mod actions;
mod keyboard;
mod modes;

pub struct Dimensions {
    pub offsets: (u16, u16),
    pub bytes: (u16, u16),
    pub decoded: (u16, u16),
}

impl Dimensions {
    pub fn new(padding: u16, parameters: &Parameters) -> Self {
        let offsets_start = padding;
        let offsets_end = offsets_start + 10;
        let offsets = (offsets_start, offsets_end);

        let bytes_start = offsets_end + 3;
        let bytes_end = (bytes_start + parameters.byte_size * 5) - 1;
        let bytes = (bytes_start, bytes_end);

        let decoded_start = bytes_end + 3;
        let decoded_end = decoded_start + parameters.byte_size;
        let decoded = (decoded_start, decoded_end);

        Self {
            bytes,
            decoded,
            offsets,
        }
    }
}

#[derive(PartialEq)]
pub enum Action {
    Quit,
    DrawBytes,
    DrawHelp,
    SkipDrawing,
}

#[derive(PartialEq)]
pub enum Direction {
    Left,
    Right,
}

pub enum StatusMode {
    General,
    Keys,
}

pub struct Parameters {
    file_path: String,
    byte_size: u16,
}

pub struct TermState<'a> {
    pub row: u16,
    pub column: u16,
    pub term_width: u16,
    pub term_height: u16,
    pub padding: u16,
    pub render_from_offset: usize,
    pub status_mode: StatusMode,
    pub dimensions: &'a Dimensions,
    pub prev_mode: Modes,
}

impl From<Args> for Parameters {
    fn from(args: Args) -> Self {
        let collected_args: Vec<String> = args.collect();
        let mut byte_size = 16;

        if collected_args.len() >= 3 {
            byte_size = collected_args[2]
                .parse()
                .expect("Second argument must be u16");
        }

        Self {
            file_path: collected_args[1].clone(),
            byte_size,
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

            match new_mode {
                modes::Modes::Bytes => index = 0,
                modes::Modes::Help => index = 1,
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

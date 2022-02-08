use std::{
    env::{self, Args},
    fs::File,
    io::{stdout, Read, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, read, Event},
    execute, queue,
    terminal::ClearType,
};
use crossterm::{terminal, Result};
use draw::{draw_bytes, draw_fixed_ui, draw_offsets};
use handlers::{handle_input, handle_mouse, handle_resize};
use keyboard::Keyboard;

mod actions;
mod draw;
mod handlers;
mod keyboard;

pub(crate) struct Dimensions {
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

pub(crate) enum Action {
    Quit,
    DrawBytes,
    DrawHelp,
    SkipDrawing,
}

#[derive(PartialEq)]
enum Direction {
    Left,
    Right,
}

enum StatusMode {
    General,
    Keys,
}

struct Parameters {
    file_path: String,
    byte_size: u16,
}

struct TermState<'a> {
    pub row: u16,
    pub column: u16,
    pub term_width: u16,
    pub term_height: u16,
    pub padding: u16,
    pub render_from_offset: usize,
    pub status_mode: StatusMode,
    pub dimensions: &'a Dimensions,
}

impl From<Args> for Parameters {
    fn from(args: Args) -> Self {
        let collected_args: Vec<String> = args.collect();
        Self {
            file_path: collected_args[1].clone(),
            byte_size: 16,
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

    let mut state = TermState {
        row: 1,
        column: dimensions.bytes.0,
        term_height: size.1,
        term_width: size.0,
        padding,
        render_from_offset: 0,
        status_mode: StatusMode::General,
        dimensions: &dimensions,
    };

    let bytes = get_bytes(&parameters.file_path)?;
    let file_size = bytes.len();

    let minimal_width = ((parameters.byte_size + 1) * 5) + 16;
    let offsets = file_size as u16 / parameters.byte_size;

    let keyboard = Keyboard::new();

    loop {
        if poll(Duration::from_millis(16))? {
            let action = match read()? {
                Event::Key(event) => handle_input(&mut state, event, &keyboard),
                Event::Mouse(event) => handle_mouse(&mut state, event),
                Event::Resize(width, height) => handle_resize(
                    &mut stdout,
                    &mut state,
                    width,
                    height,
                    minimal_width,
                    &parameters,
                )?,
            };

            match action {
                Action::Quit => break,
                Action::SkipDrawing => continue,
                _ => {}
            }

            queue!(&mut stdout, terminal::Clear(ClearType::All))?;

            draw_fixed_ui(&mut stdout, &state, &parameters, &keyboard)?;
            draw_offsets(&mut stdout, &state, &parameters, offsets)?;
            draw_bytes(&mut stdout, &state, &parameters, &bytes)?;

            queue!(&mut stdout, cursor::MoveTo(state.column, state.row))?;

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

fn get_bytes(path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path).expect("Failed to open file");
    let file_size = file.metadata()?.len();
    let mut bytes: Vec<u8> = vec![0; file_size as usize];
    file.read(&mut bytes)
        .expect("Failed to read bytes into buffer");

    Ok(bytes)
}

use std::{
    collections::HashMap,
    env::{self, Args},
    fs::File,
    io::{stdout, Read, Stdout, Write},
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
use keyboard::Shortcuts;

mod keyboard;

struct Parameters {
    file_path: String,
    byte_size: u16,
}
struct TermState {
    pub row: u16,
    pub column: u16,
    pub term_width: u16,
    pub term_height: u16,
    pub padding: u16,
    pub render_from_offset: usize,
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

    let mut state = TermState {
        row: 1,
        column: 1,
        term_height: size.1,
        term_width: size.0,
        padding: 2,
        render_from_offset: 0,
    };

    let mut file = File::open(&parameters.file_path).expect("Failed to open file");
    let file_size = file.metadata()?.len();
    let mut bytes: Vec<u8> = vec![0; file_size as usize];
    file.read(&mut bytes)
        .expect("Failed to read bytes into buffer");

    let minimal_width = ((parameters.byte_size + 1) * 5) + 16;
    let offsets = file_size as u16 / parameters.byte_size;

    let shortcuts = Shortcuts::new();

    loop {
        if poll(Duration::from_millis(100))? {
            let code = match read()? {
                Event::Key(event) => handle_input(&mut state, event, &shortcuts),
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

            //@Improvement: Change to enum
            if code == 2 {
                continue;
            }

            if code == 1 {
                break;
            }

            if state.column > state.term_width {
                state.column = state.term_width
            }
            if state.row > state.term_height {
                state.row = state.term_height
            }

            queue!(&mut stdout, terminal::Clear(ClearType::All))?;

            draw_fixed_ui(&mut stdout, &state, &parameters)?;
            draw_offsets(&mut stdout, &state, &parameters, offsets)?;
            draw_bytes(&mut stdout, &state, &parameters, &bytes)?;

            //Update cursor position
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

fn handle_resize(
    stdout: &mut Stdout,
    state: &mut TermState,
    width: u16,
    height: u16,
    minimal_width: u16,
    parameters: &Parameters,
) -> Result<u8> {
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

        return Ok(2);
    }

    state.term_width = width;
    state.term_height = height;

    Ok(0)
}

fn draw_offsets(
    stdout: &mut Stdout,
    state: &TermState,
    parameters: &Parameters,
    offsets: u16,
) -> Result<()> {
    let mut iter = 0;
    for i in state.render_from_offset as u16..offsets + 1 {
        if iter >= state.term_height - 1 {
            break;
        }
        queue!(
            stdout,
            style::SetForegroundColor(Color::Yellow),
            cursor::MoveTo(state.padding, iter + 1 as u16),
            style::Print(format!("{:#010x}", i * parameters.byte_size))
        )?;
        iter += 1;
    }
    Ok(())
}

fn draw_bytes(
    stdout: &mut Stdout,
    state: &TermState,
    parameters: &Parameters,
    bytes: &Vec<u8>,
) -> Result<()> {
    //For each byte in file
    let mut byte_x = state.padding + 13;
    let mut byte_y = 1;
    queue!(stdout, style::SetForegroundColor(Color::DarkBlue))?;

    let mut iter = 0;
    let start_from = parameters.byte_size as usize * state.render_from_offset;

    for i in start_from..bytes.len() {
        let byte = bytes[i];
        queue!(
            stdout,
            cursor::MoveTo(byte_x, byte_y),
            style::Print(format!("{:#04X}", byte))
        )?;

        byte_x += 5;
        iter += 1;

        //Overflow on x axis
        if iter >= parameters.byte_size || i == bytes.len() - 1 {
            queue!(stdout, cursor::MoveTo(97, byte_y))?;
            for j in i + 1 - iter as usize..=i {
                let decoded = get_symbol(bytes[j]);
                queue!(stdout, cursor::MoveRight(0), style::Print(decoded))?;
            }
            iter = 0;
            byte_x = state.padding + 13;
            byte_y += 1;
        }

        //Overflow on y axis (columns)
        if byte_y >= state.term_height {
            break;
        }
    }
    Ok(())
}

fn get_symbol(byte: u8) -> char {
    if byte.is_ascii_whitespace() {
        return ' ';
    }

    if !byte.is_ascii() || byte.is_ascii_control() {
        return '.';
    }

    char::from(byte)
}

fn draw_fixed_ui<W: Write>(
    stdout: &mut W,
    state: &TermState,
    parameters: &Parameters,
) -> Result<()> {
    let status = format!(
        "Hex Editor ({}x{}) - {}:{}, file: {}",
        state.term_width, state.term_height, state.column, state.row, &parameters.file_path
    );
    //Status
    queue!(
        stdout,
        style::SetForegroundColor(Color::Yellow),
        cursor::MoveTo(state.padding, 0),
        style::Print("Offset(h)"),
        cursor::MoveTo(state.padding, state.term_height),
        style::Print(status),
        cursor::MoveTo(state.padding + 12, 0),
    )?;

    //Byte columns
    for i in 0..parameters.byte_size {
        queue!(
            stdout,
            cursor::MoveRight(1),
            style::Print(format!("{:#04X}", i))
        )?;
    }
    //Decoded
    queue!(stdout, cursor::MoveRight(3), style::Print("Decoded"))?;
    Ok(())
}

fn handle_mouse(state: &mut TermState, event: event::MouseEvent) -> u8 {
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
    0
}

fn handle_input(state: &mut TermState, event: KeyEvent, shortcuts: &Shortcuts) -> u8 {
    match shortcuts.get(&event.code) {
        Some(action) => action(state),
        None => 0,
    }
}

use std::{
    env::{self, Args},
    fs::File,
    io::{self, stdout, Read, Stdout, Write},
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

    let file = File::open(&parameters.file_path).expect("Failed to open file");
    let file_size = file.metadata()?.len();
    let bytes: Vec<Result<u8>> = file.bytes().collect();
    let offset_txt = "Offset(h)";
    let minimal_width = ((parameters.byte_size + 1) * 5) + offset_txt.len() as u16;

    loop {
        if poll(Duration::from_millis(100))? {
            let code = match read()? {
                Event::Key(event) => handle_input(&mut state, event),
                Event::Resize(width, height) => {
                    if width < minimal_width {
                        queue!(
                            &mut stdout,
                            terminal::Clear(ClearType::All),
                            cursor::MoveTo(1, 1),
                            style::Print(format!(
                                "Windows too small to display {} bytes in one row",
                                parameters.byte_size
                            ))
                        )?;
                        stdout.flush()?;
                        continue;
                    }

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
                "Hex Editor ({}x{}) - {}:{}, file: {}",
                state.term_width, state.term_height, state.column, state.row, &parameters.file_path
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
                cursor::MoveTo(state.padding, 0),
                style::Print(offset_txt),
                cursor::MoveTo(state.padding, state.term_height),
                style::Print(status),
            )?;

            let margin = (offset_txt.len() + 6) as u16;

            //Render byte columns
            for i in 0..parameters.byte_size {
                queue!(
                    &mut stdout,
                    cursor::MoveTo(margin + i * 5, 0),
                    style::Print(format!("{:#04X}", i))
                )?;
            }

            let how_many_to_skip = parameters.byte_size as usize * state.render_from_offset;
            //For each offset
            let file_size = file_size as u16;
            let offsets = file_size / parameters.byte_size;
            let mut iter = 0;
            for i in state.render_from_offset as u16..offsets {
                if iter >= state.term_height - 1 {
                    break;
                }
                queue!(
                    &mut stdout,
                    style::SetForegroundColor(Color::Yellow),
                    cursor::MoveTo(state.padding, iter + 1 as u16),
                    style::Print(format!("{:#010x}", i * parameters.byte_size))
                )?;
                iter += 1;
            }

            //For each byte in file
            let mut byte_x = state.padding + 13;
            let mut byte_y = 1;
            queue!(
                &mut stdout,
                cursor::MoveTo(byte_x, byte_y),
                style::SetForegroundColor(Color::DarkBlue)
            )?;
            let mut iter = 0;
            for i in how_many_to_skip..bytes.len() {
                let possible_byte = &bytes[i];
                match possible_byte {
                    Ok(byte) => {
                        queue!(
                            &mut stdout,
                            cursor::MoveTo(byte_x, byte_y),
                            style::Print(format!("{:#04X}", byte))
                        )?;

                        byte_x += 5;
                        iter += 1;

                        //Overflow on x axis
                        if iter >= parameters.byte_size {
                            iter = 0;
                            byte_x = state.padding + 13;
                            byte_y += 1;
                        }

                        //Overflow on y axis (columns)
                        if byte_y >= state.term_height {
                            break;
                        }
                    }
                    Err(_) => panic!("Failed to read byte number: {}", iter),
                }
            }

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
        KeyCode::PageUp => {
            if state.render_from_offset != 0 {
                state.render_from_offset -= 1
            }
        }
        KeyCode::PageDown => state.render_from_offset += 1,
        KeyCode::Char(char) => match char {
            'q' => return 1,
            _ => return 0,
        },
        _ => {}
    }
    0
}
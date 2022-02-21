use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
};

use crate::{
    misc::get_byte_at_cursor,
    misc::{Direction, Parameters},
    modes::Modes,
    StatusMode, TermState,
};

pub fn general_status(state: &mut TermState, parameters: &Parameters) -> Modes {
    state.status_mode = StatusMode::General;
    Modes::Bytes
}

pub fn keys_status(state: &mut TermState, parameters: &Parameters) -> Modes {
    state.status_mode = StatusMode::Keys;
    Modes::Bytes
}

pub fn help(state: &mut TermState, parameters: &Parameters) -> Modes {
    if state.prev_mode != Modes::Help {
        return Modes::Help;
    }

    Modes::Bytes
}

pub fn remove(state: &mut TermState, parameters: &Parameters) -> Modes {
    let byte_index = get_byte_at_cursor(state, parameters);

    if state.bytes_changed.contains(&byte_index) {
        state.bytes_changed.remove(&byte_index);
    }

    if !state.bytes_removed.contains(&byte_index) {
        state.bytes_removed.insert(byte_index);
    } else {
        state.bytes_removed.remove(&byte_index);
    }

    Modes::Bytes
}

pub fn save(state: &mut TermState, parameters: &Parameters) -> Modes {
    //@Improve: Change this to some sort of Rope data structure in the future.
    let mut bytes_copy = Vec::with_capacity(state.bytes.len());
    for i in 0..state.bytes.len() {
        if state.bytes_removed.contains(&i) {
            continue;
        }
        bytes_copy.push(state.bytes[i]);
    }
    state.bytes = bytes_copy;
    state.bytes_removed.clear();

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&parameters.file_path)
        .expect("Failed to save changes");

    file.write(&state.bytes).expect("Failed to save changes");
    state.bytes_changed.clear();

    Modes::Bytes
}

pub fn edit(state: &mut TermState, parameters: &Parameters) -> Modes {
    Modes::Change
}

pub fn go_left(state: &mut TermState, parameters: &Parameters) -> Modes {
    let jump_by = calculate_leap(&state, Direction::Left);
    if jump_by <= state.column {
        state.column -= jump_by;
    }
    Modes::Bytes
}

pub fn go_right(state: &mut TermState, parameters: &Parameters) -> Modes {
    let jump_by = calculate_leap(&state, Direction::Right);
    if state.column + jump_by <= state.term_width {
        state.column += jump_by;
    }
    Modes::Bytes
}

pub fn go_up(state: &mut TermState, parameters: &Parameters) -> Modes {
    if state.row >= 2 {
        state.row -= 1;
    }
    Modes::Bytes
}

pub fn go_down(state: &mut TermState, parameters: &Parameters) -> Modes {
    if state.row != state.term_height {
        state.row += 1;
    }
    Modes::Bytes
}

pub fn scroll_up(state: &mut TermState, parameters: &Parameters) -> Modes {
    if state.render_from_offset != 0 {
        state.render_from_offset -= 1
    }
    Modes::Bytes
}

pub fn scroll_down(state: &mut TermState, parameters: &Parameters) -> Modes {
    state.render_from_offset += 1;
    Modes::Bytes
}

pub fn quit(state: &mut TermState, parameters: &Parameters) -> Modes {
    Modes::Quit
}

fn calculate_leap(state: &TermState, direction: Direction) -> u16 {
    let dimensions = state.dimensions;

    //Do not allow jump into offsets
    if direction == Direction::Left && state.column == dimensions.bytes.0 {
        return 0;
    }

    //Jumping from last byte onto first char of decode section
    if state.column == dimensions.bytes.1 - 4 && direction == Direction::Right {
        return 7;
    }

    //Jumping from decode to last byte
    if state.column == dimensions.decoded.0 && Direction::Left == direction {
        return 7;
    }

    //Jumping between bytes
    if state.column >= dimensions.bytes.0 && state.column <= dimensions.bytes.1 {
        return 5;
    }

    1
}

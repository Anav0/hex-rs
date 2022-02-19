use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use crate::{
    misc::{get_byte_at_cursor, Action},
    misc::{Direction, Parameters},
    modes::Modes,
    StatusMode, TermState,
};

fn save_bytes(path: &str, bytes: &Vec<u8>) {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Failed to save changes");

    file.write(&bytes).expect("Failed to save changes");
}

pub fn general_status(state: &mut TermState, parameters: &Parameters) -> Action {
    state.status_mode = StatusMode::General;
    Action::DrawBytes
}

pub fn keys_status(state: &mut TermState, parameters: &Parameters) -> Action {
    state.status_mode = StatusMode::Keys;
    Action::DrawBytes
}

pub fn help(state: &mut TermState, parameters: &Parameters) -> Action {
    if state.prev_mode != Modes::Help {
        return Action::DrawHelp;
    }

    Action::DrawBytes
}

pub fn remove(state: &mut TermState, parameters: &Parameters) -> Action {
    let byte_index = get_byte_at_cursor(state, parameters);

    state.bytes.remove(byte_index);

    save_bytes(state.file_path, &state.bytes);

    state.bytes_changed.clear();

    Action::DrawBytes
}

pub fn save(state: &mut TermState, parameters: &Parameters) -> Action {
    save_bytes(state.file_path, &state.bytes);

    state.bytes_changed.clear();

    Action::DrawBytes
}

pub fn edit(state: &mut TermState, parameters: &Parameters) -> Action {
    Action::Change
}

pub fn go_left(state: &mut TermState, parameters: &Parameters) -> Action {
    let jump_by = calculate_leap(&state, Direction::Left);
    if jump_by <= state.column {
        state.column -= jump_by;
    }
    Action::DrawBytes
}

pub fn go_right(state: &mut TermState, parameters: &Parameters) -> Action {
    let jump_by = calculate_leap(&state, Direction::Right);
    if state.column + jump_by <= state.term_width {
        state.column += jump_by;
    }
    Action::DrawBytes
}

pub fn go_up(state: &mut TermState, parameters: &Parameters) -> Action {
    if state.row >= 2 {
        state.row -= 1;
    }
    Action::DrawBytes
}

pub fn go_down(state: &mut TermState, parameters: &Parameters) -> Action {
    if state.row != state.term_height {
        state.row += 1;
    }
    Action::DrawBytes
}

pub fn scroll_up(state: &mut TermState, parameters: &Parameters) -> Action {
    if state.render_from_offset != 0 {
        state.render_from_offset -= 1
    }
    Action::DrawBytes
}

pub fn scroll_down(state: &mut TermState, parameters: &Parameters) -> Action {
    state.render_from_offset += 1;
    Action::DrawBytes
}

pub fn quit(state: &mut TermState, parameters: &Parameters) -> Action {
    Action::Quit
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

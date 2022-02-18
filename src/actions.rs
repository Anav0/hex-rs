use crate::{misc::Action, misc::Direction, modes::Modes, StatusMode, TermState};

pub fn general_status(state: &mut TermState) -> Action {
    state.status_mode = StatusMode::General;
    Action::DrawBytes
}

pub fn keys_status(state: &mut TermState) -> Action {
    state.status_mode = StatusMode::Keys;
    Action::DrawBytes
}

pub fn help(state: &mut TermState) -> Action {
    if state.prev_mode != Modes::Help {
        return Action::DrawHelp;
    }

    Action::DrawBytes
}

pub fn remove(state: &mut TermState) -> Action {
    Action::DrawBytes
}

pub fn save(state: &mut TermState) -> Action {
    Action::DrawBytes
}

pub fn edit(state: &mut TermState) -> Action {
    Action::Change
}

pub fn delete(state: &mut TermState) -> Action {
    Action::DrawBytes
}

pub fn go_left(state: &mut TermState) -> Action {
    let jump_by = calculate_leap(&state, Direction::Left);
    if jump_by <= state.column {
        state.column -= jump_by;
    }
    Action::DrawBytes
}

pub fn go_right(state: &mut TermState) -> Action {
    let jump_by = calculate_leap(&state, Direction::Right);
    if state.column + jump_by <= state.term_width {
        state.column += jump_by;
    }
    Action::DrawBytes
}

pub fn go_up(state: &mut TermState) -> Action {
    if state.row >= 2 {
        state.row -= 1;
    }
    Action::DrawBytes
}

pub fn go_down(state: &mut TermState) -> Action {
    if state.row != state.term_height {
        state.row += 1;
    }
    Action::DrawBytes
}

pub fn scroll_up(state: &mut TermState) -> Action {
    if state.render_from_offset != 0 {
        state.render_from_offset -= 1
    }
    Action::DrawBytes
}

pub fn scroll_down(state: &mut TermState) -> Action {
    state.render_from_offset += 1;
    Action::DrawBytes
}

pub fn quit(state: &mut TermState) -> Action {
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

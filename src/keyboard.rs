#![allow(dead_code)]

use crossterm::{event::KeyCode, style::Stylize};
use directories::ProjectDirs;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use crate::TermState;

pub(crate) type KeyAction = dyn Fn(&mut TermState) -> u8;

pub(crate) struct Shortcuts<'a> {
    pub pairs: HashMap<KeyCode, &'a KeyAction>,
    help: Vec<String>,
}
impl<'a> Shortcuts<'a> {
    pub fn new() -> Self {
        let config_path = ProjectDirs::from("com", "Papilionem", "Hex editor")
            .expect("Failed to create config path");

        let config_dir = config_path.config_dir().to_path_buf();

        let keys_path = match config_dir.exists() {
            true => {
                let mut key_path = config_dir.clone();
                key_path.push("keys");
                key_path
            }
            false => create_config(&config_dir),
        };

        let mut pairs: HashMap<KeyCode, &'a KeyAction> = HashMap::new();
        let mut help: Vec<String> = vec![];

        let file = File::open(keys_path).expect("Failed to open config file");
        let reader = BufReader::new(file);

        let mut iter = 0;
        for line in reader.lines() {
            if line.is_err() {
                panic!("Failed to read line: '{}' from config file", iter + 1)
            }

            let line_string = line.unwrap();
            let line_string_trimed = line_string.trim();

            let splited: Vec<&str> = line_string_trimed.split_whitespace().collect();

            let key = splited[0];
            let action = *splited.last().unwrap();

            let matched_key: KeyCode = match_key(key);
            let matched_action: &KeyAction = match_action(action);

            pairs.insert(matched_key, matched_action);
            help.push(format!("{}: {}", key, action));

            iter += 1;
        }
        Self { pairs, help }
    }

    pub fn get(&self, code: &KeyCode) -> Option<&&KeyAction> {
        self.pairs.get(&code)
    }

    pub fn help(&self) -> String {
        self.help.join(" ")
    }
}
fn match_key(key: &str) -> KeyCode {
    let uniform_key = key.to_lowercase();

    let is_f_key = uniform_key.starts_with("f");
    let is_char = uniform_key.len() == 1;

    if is_char {
        let chars: Vec<char> = uniform_key.chars().collect();
        return KeyCode::Char(chars[0]);
    }

    if is_f_key {
        let chars: Vec<&str> = uniform_key.split("f").collect();
        if chars.len() > 2 {
            panic!("Failed to parse key: '{}'", key);
        }

        let number = chars[1]
            .parse::<u8>()
            .expect(&format!("Failed to parse key: '{}'", key));

        if number > 12 || number < 1 {
            panic!("Failed to parse key: '{}'", key);
        }

        return KeyCode::F(number);
    }

    match uniform_key.as_str() {
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "pg_up" => KeyCode::PageUp,
        "pg_down" => KeyCode::PageDown,
        "enter" => KeyCode::Enter,
        "end" => KeyCode::End,
        "home" => KeyCode::Home,
        "insert" => KeyCode::Insert,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "esc" => KeyCode::Esc,
        "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        _ => panic!("Unrecognized key: '{}'", key),
    }
}
fn match_action<'b>(action: &str) -> &'b KeyAction {
    match action {
        "go_left" => &go_left,
        "go_right" => &go_right,
        "go_down" => &go_down,
        "go_up" => &go_up,
        "scroll_down" => &scroll_down,
        "scroll_up" => &scroll_up,
        "quit" => &quit,
        "exit" => &quit,
        "delete" => &delete,
        "edit" => &edit,
        "remove" => &remove,
        "save" => &save,
        "help" => &help,
        _ => panic!("Unrecognized action: '{}'", action),
    }
}
pub fn create_config(path: &PathBuf) -> PathBuf {
    fs::create_dir_all(path).expect(&format!("Failed to create config dir: '{:?}'", &path));

    let mut key_path = path.clone();

    key_path.push("keys");

    let mut file = File::create(&key_path).expect(&format!(
        "Failed to create default config file at: {:?}",
        &key_path
    ));

    let mut keys = String::from("");
    keys += "left    go_left\n";
    keys += "right   go_right\n";
    keys += "up      left go_up\n";
    keys += "pg_up   scroll_up\n";
    keys += "pg_down scroll_down\n";
    keys += "q       quit\n";
    keys += "f1      help\n";
    keys += "f2      edit\n";
    keys += "f3      remove\n";
    keys += "f5      save\n";

    file.write_all(keys.as_bytes())
        .expect("Failed to write to keys config file");

    key_path
}

fn help(state: &mut TermState) -> u8 {
    0
}

fn remove(state: &mut TermState) -> u8 {
    0
}

fn save(state: &mut TermState) -> u8 {
    0
}

fn edit(state: &mut TermState) -> u8 {
    0
}

fn delete(state: &mut TermState) -> u8 {
    0
}

fn go_left(state: &mut TermState) -> u8 {
    if state.column != 0 {
        state.column -= 1;
    }
    0
}

fn go_right(state: &mut TermState) -> u8 {
    if state.column != state.term_width {
        state.column += 1;
    }
    0
}

fn go_up(state: &mut TermState) -> u8 {
    if state.row != 0 {
        state.row -= 1;
    }
    0
}

fn go_down(state: &mut TermState) -> u8 {
    if state.row != state.term_height {
        state.row += 1;
    }
    0
}

fn scroll_up(state: &mut TermState) -> u8 {
    if state.render_from_offset != 0 {
        state.render_from_offset -= 1
    }
    0
}
fn scroll_down(state: &mut TermState) -> u8 {
    state.render_from_offset += 1;
    0
}
fn quit(state: &mut TermState) -> u8 {
    1
}

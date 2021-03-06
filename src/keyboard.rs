#![allow(dead_code)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use crate::{
    actions::{
        edit, general_status, go_down, go_left, go_right, go_to_end, go_to_start, go_up, help,
        next_change, next_found, prev_change, prev_found, quit, remove, save, scroll_down,
        scroll_up, search,
    },
    misc::Parameters,
    modes::Modes,
    TermState,
};

pub type KeyAction = dyn Fn(&mut TermState, &Parameters) -> Modes;

pub struct Keyboard<'a> {
    keys_and_actions: HashMap<KeyEvent, &'a KeyAction>,
    help: Vec<String>,
}
impl<'a> Keyboard<'a> {
    pub fn new() -> Self {
        let config_path = ProjectDirs::from("com", "Papilionem", "Hex editor")
            .expect("Failed to create config path");

        let config_dir = config_path.config_dir().to_path_buf();

        let mut key_path = config_dir.clone();
        key_path.push("keys");

        if !key_path.exists() {
            create_config(&config_dir);
        }

        let mut pairs: HashMap<KeyEvent, &'a KeyAction> = HashMap::new();
        let mut help: Vec<String> = vec![];

        let file = File::open(key_path).expect("Failed to open config file");
        let reader = BufReader::new(file);

        let mut iter = 0;
        for line in reader.lines() {
            if line.is_err() {
                panic!("Failed to read line: '{}' from config file", iter + 1)
            }

            let line_string = line.unwrap();
            let line_string_trimed = line_string.trim();

            let splited: Vec<&str> = line_string_trimed.split_whitespace().collect();

            let mut key_str = splited[0];
            let action = *splited.last().unwrap();

            let mut modifier: KeyModifiers = KeyModifiers::NONE;

            // We have a modifier to parse
            if key_str.contains("+") {
                let splited: Vec<&str> = key_str.split("+").collect();
                if splited.len() != 2 {
                    panic!("Failed to parse keys entry: '{}'", key_str);
                }
                modifier = match_modifier(splited[0]).unwrap();
                key_str = splited[1];
            }

            let matched_key: KeyCode = match_key(key_str);
            let event = KeyEvent::new(matched_key, modifier);
            let (matched_action, desc) = match_action(action);

            pairs.insert(event, matched_action);
            help.push(format!("{}: {}", key_str, desc));

            iter += 1;
        }
        Self {
            keys_and_actions: pairs,
            help,
        }
    }

    pub fn get(&self, code: &KeyEvent) -> Option<&&KeyAction> {
        self.keys_and_actions.get(&code)
    }

    pub fn help(&self, separator: &str) -> String {
        self.help.join(separator)
    }
}

fn match_modifier(modifier: &str) -> Result<KeyModifiers, String> {
    let uniform = modifier.trim().to_lowercase();
    let uniform_str = uniform.as_str();

    match uniform_str {
        "control" | "ctrl" => Ok(KeyModifiers::CONTROL),
        "shift" => Ok(KeyModifiers::SHIFT),
        "alt" => Ok(KeyModifiers::ALT),
        _ => Err(format!("Unrecognized modifier: '{}'", modifier)),
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
        "down" => KeyCode::Down,
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

fn match_action<'b>(action: &str) -> (&'b KeyAction, &str) {
    match action {
        "go_left" => (&go_left, "moves cursor to the previous element"),
        "go_right" => (&go_right, "moves cursor to the next element"),
        "go_down" => (&go_down, "moves cursor down an offset"),
        "go_up" => (&go_up, "moves cursor up an offset"),
        "scroll_down" => (&scroll_down, "next offset"),
        "scroll_up" => (&scroll_up, "pervious offset"),
        "quit" => (&quit, "quit"),
        "exit" => (&quit, "quit"),
        "goto" => (&|_, _| Modes::GoTo, "Go to"),
        "delete" => (&remove, "Remove byte"),
        "edit" => (&edit, "Change byte"),
        "save" => (&save, "Save changes"),
        "help" => (&help, "Print help"),
        "next_change" => (&next_change, "Goes to next change"),
        "prev_change" => (&prev_change, "Goes to previous change"),
        "next_found" => (&next_found, "Goes to next found sequence"),
        "prev_found" => (&prev_found, "Goes to previous found sequence"),
        "go_to_start" => (&go_to_start, "Goes to first offset"),
        "go_to_end" => (&go_to_end, "Goes to last offset"),
        "general_status" => (&general_status, "changes status bar to its general state"),
        "search" => (&search, "Search for sequence"),
        _ => panic!("Unrecognized action: '{}'", action),
    }
}

fn create_config(path: &PathBuf) -> PathBuf {
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
    keys += "down    left go_down\n";
    keys += "pg_up   scroll_up\n";
    keys += "pg_down scroll_down\n";
    keys += "q       quit\n";
    keys += "h       help\n";
    keys += "f2      edit\n";
    keys += "f3      delete\n";
    keys += "f5      save\n";
    keys += "1       general_status\n";
    keys += "shift+: goto\n";
    keys += "n       next_change\n";
    keys += "ctrl+n  next_found\n";
    keys += "p       prev_change\n";
    keys += "ctrl+p  prev_found\n";
    keys += "f       search\n";
    keys += "home    go_to_start\n";
    keys += "end     go_to_end\n";

    file.write_all(keys.as_bytes())
        .expect("Failed to write to keys config file");

    key_path
}

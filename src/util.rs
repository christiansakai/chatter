use std::process::Command;
use std::time::Duration;
use std::thread;
use std::io::{self, Write};

pub fn clear() {
    Command::new("clear").status().unwrap();
}

pub fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

pub enum Input {
    Message(String),
    Quit,
}

pub fn get_user_input() -> Input {
    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .unwrap();

    let message = buffer.trim().to_string();

    if message == ":q" {
        return Input::Quit;
    }

    Input::Message(message)
}

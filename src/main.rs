use std::io::{self, Read};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() {
    enable_raw_mode().unwrap();
    for b in io::stdin().bytes() {
        let char = b.unwrap() as char;
        println!("{}\r", char);
        if char == 'q' {
            disable_raw_mode().unwrap();
            break;
        }
    }
}

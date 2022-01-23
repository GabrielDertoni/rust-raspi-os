#![allow(dead_code)]

use std::process::Command;

const RESET:  &str = "\x1b[0m";
const GREEN:  &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE:   &str  = "\x1b[34m";

pub fn print_info(s: impl AsRef<str>) {
    println!("\t{}[INFO]{}\t{}", BLUE, RESET, s.as_ref());
}

pub fn print_command(cmd: &Command) {
    print!(
        "\t{}[CMD]{}\t{}",
        GREEN,
        RESET,
        (cmd.get_program().as_ref() as &std::path::Path)
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    for arg in cmd.get_args() {
        print!(" {}", arg.to_string_lossy());
    }
    println!();
}

use crate::version::{PROGRAM, VERSION};
use colored::*;

/// Pretty print the version number.
pub fn display_version() {
    println!("{}", format!("{PROGRAM} {VERSION}").yellow());
}

/// Pretty print an action. "Level" defines color and indentation.
pub fn display_action(action: &str, level: u32) {
    for _ in 0..level {
        print!(" ");
    }
    print!(" - ");
    match level {
        1 => println!("{}", action.cyan()),
        2 => println!("{}", action.bright_blue()),
        3 => println!("{}", action.magenta()),
        _ => println!("{}", action.white()),
    }
}

/// Display that a task succeeded. Level defines indentation.
pub fn display_success(action: &str, level: u32) {
    for _ in 0..level {
        print!(" ");
    }
    println!(" ✓ {}", action.green());
}

/// Display that a task failed. Level defines indentation.
pub fn display_error(action: &str, level: u32) {
    for _ in 0..level {
        print!(" ");
    }
    println!(" X {}", action.red());
}

/// Display that a task generated a warning. Level defines indentation.
pub fn display_warning(action: &str, level: u32) {
    for _ in 0..level {
        print!(" ");
    }
    println!(" ? {}", action.magenta());
}

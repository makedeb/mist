use crate::style::Colorize;

pub fn info(string: &str) {
    print!("{} {}", "Info:".cyan().bold(), string);
}

pub fn warning(string: &str) {
    print!("{} {}", "Warning:".yellow().bold(), string);
}

pub fn error(string: &str) {
    print!("{} {}", "Err:".red().bold(), string);
}

pub fn question(string: &str) {
    print!("{} {}", "Question:".magenta().bold(), string);
}

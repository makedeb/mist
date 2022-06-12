use ansi_term::{Colour, Style};

pub fn info(str: &str) {
    println!("{}", str);
}

pub fn error(str: &str) {
    println!("{} {}", Colour::Red.paint("Err:"), str);
}

pub fn error_bold(str: &str) {
    println!(
        "{} {}",
        Colour::Red.paint("Err:"),
        Style::new().bold().paint(str)
    );
}

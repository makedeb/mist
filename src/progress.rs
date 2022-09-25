use crate::{
    apt_util::{self, NumSys},
    style::Colorize,
};
use rust_apt::progress::{AcquireProgress, InstallProgress, Worker};
use std::io::{self, Write};

/// Acquire progress struct.
pub struct MistAcquireProgress {}

impl AcquireProgress for MistAcquireProgress {
    fn pulse_interval(&self) -> usize {
        500000
    }

    fn hit(&mut self, id: u32, description: String) {
        println!(
            "{}{} {}",
            "Hit:".green().bold(),
            id.to_string().green().bold(),
            description
        );
    }

    fn fetch(&mut self, id: u32, description: String, _file_size: u64) {
        println!(
            "{}{} {}",
            "Get:".green().bold(),
            id.to_string().green().bold(),
            description
        );
    }

    fn fail(&mut self, id: u32, description: String, status: u32, error_text: String) {
        if status == 0 || status == 2 {
            println!(
                "{} {}",
                format!("{}{} ({})", "Ign:", id, error_text).yellow().bold(),
                description
            );
        } else {
            println!(
                "{} {}",
                format!("{}{} ({})", "Err:", id, error_text).red().bold(),
                description
            );
        }
    }

    fn pulse(
        &mut self,
        _workers: Vec<Worker>,
        _percent: f32,
        _total_bytes: u64,
        _current_bytes: u64,
        _current_cps: u64,
    ) {
    }

    fn done(&mut self) {}

    fn start(&mut self) {}

    fn stop(
        &mut self,
        fetched_bytes: u64,
        elapsed_time: u64,
        current_cps: u64,
        _pending_errors: bool,
    ) {
        if fetched_bytes == 0 {
            return;
        }

        println!(
            "{}",
            format!(
                "Fetched {} in {} ({}/s)",
                apt_util::unit_str(fetched_bytes, NumSys::Decimal),
                apt_util::time_str(elapsed_time),
                apt_util::unit_str(current_cps, NumSys::Decimal)
            )
            .bold()
        )
    }
}

/// Install progress struct.
pub struct MistInstallProgress {}

impl InstallProgress for MistInstallProgress {
    fn status_changed(
        &mut self,
        _pkgname: String,
        steps_done: u64,
        total_steps: u64,
        _action: String,
    ) {
        // Get the terminal's width and height.
        let term_height = apt_util::terminal_height();
        let term_width = apt_util::terminal_width();

        // Save the current cursor position.
        print!("\x1b7");

        // Go to the progress reporting line.
        print!("\x1b[{};0f", term_height);
        io::stdout().flush().unwrap();

        // Convert the float to a percentage string.
        let percent = steps_done as f32 / total_steps as f32;
        let mut percent_str = (percent * 100.0).round().to_string();

        let percent_padding = match percent_str.len() {
            1 => "  ",
            2 => " ",
            3 => "",
            _ => unreachable!(),
        };

        percent_str = percent_padding.to_owned() + &percent_str;

        print!(
            "{}",
            format!("Progress: [{}{}] ", percent_str.blue(), "%".blue()).bold()
        );

        // The length of "Progress: [100%] ".
        const PROGRESS_STR_LEN: usize = 17;

        // Print the progress bar.
        // We should safely be able to convert the `usize`.try_into() into the `u32`
        // needed by `get_apt_progress_string`, as usize ints only take up 8 bytes on a
        // 64-bit processor.
        print!(
            "{}",
            apt_util::get_apt_progress_string(
                percent,
                (term_width - PROGRESS_STR_LEN).try_into().unwrap()
            )
            .bold()
        );
        io::stdout().flush().unwrap();

        // Finally, go back to the previous cursor position.
        print!("\x1b8");
        io::stdout().flush().unwrap();
    }

    fn error(&mut self, _pkgname: String, _steps_done: u64, _total_steps: u64, _error: String) {}
}

use crate::color::Colorize;
use rust_apt::{
    progress::{InstallProgress, UpdateProgress, Worker},
    util::{time_str, unit_str, NumSys},
};

/// Acquire progress struct.
pub struct MistAcquireProgress {}

impl UpdateProgress for MistAcquireProgress {
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
                format!("{}{} ({})", "Err:", id, error_text).yellow().bold(),
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
                unit_str(fetched_bytes, NumSys::Decimal),
                time_str(elapsed_time),
                unit_str(current_cps, NumSys::Decimal)
            )
            .bold()
        )
    }
}

/// Install progress struct.
pub struct MistInstallProgress {}

impl InstallProgress for MistInstallProgress {
    fn inst_status_changed(
        &mut self,
        _pkgname: String,
        _steps_done: u64,
        _total_steps: u64,
        _percent: f32,
        _action: String,
    ) {
    }
    fn inst_error(
        &mut self,
        _pkgname: String,
        _steps_done: u64,
        _total_steps: u64,
        _percent: f32,
        _error: String,
    ) {
    }
}

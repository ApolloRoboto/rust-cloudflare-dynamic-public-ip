use log::{Level, Log, Metadata, Record};

use colored::Colorize;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

pub struct SimpleLogger;

pub static LOGGER: SimpleLogger = SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with(&CRATE_NAME.replace("-", "_"))
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level_text = match record.level() {
            Level::Error => "ERROR".bright_red(),
            Level::Warn => "WARNING".yellow(),
            Level::Info => "INFO".green(),
            Level::Debug => "DEBUG".blue(),
            Level::Trace => "TRACE".cyan(),
        };

        let now = chrono::Local::now();
        let now_text = now
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            .bright_black();

        for (i, line) in record.args().to_string().lines().enumerate() {
            if i == 0 {
                println!("{:<26} {:<8} {}", now_text, level_text, line);
            } else {
                println!("{:<26} {:<8} {}", "", "", line);
            }
        }
    }

    fn flush(&self) {}
}

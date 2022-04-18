use std::fs;
use std::io::prelude::*;

use colored::*;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

#[derive(Debug)]
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let file_path = match record.file() {
                Some(path) => match path.split_once(".cargo") {
                    // cut down to only lib name
                    Some((_, path)) => &path[42..],
                    None => path,
                },
                None => "<unknown>",
            };

            // if it's from a dependency only log debug and above, else everything
            if !record.file().unwrap_or("").contains(".cargo") || record.level() <= Level::Debug {
                let level = match record.level() {
                    Level::Error => "ERROR".red(),
                    Level::Warn => "WARN".yellow(),
                    Level::Info => "INFO".green(),
                    Level::Debug => "DEBUG".cyan(),
                    Level::Trace => "TRACE".blue(),
                };

                println!(
                    "{}{:<5} {}:{}{} {}",
                    "[".truecolor(100, 100, 100),
                    level,
                    file_path,
                    record.line().unwrap_or(0),
                    "]".truecolor(100, 100, 100),
                    record.args()
                );
            }

            // we need unsafe to write to a global variable
            unsafe {
                let level = match record.level() {
                    Level::Error => "ERROR",
                    Level::Warn => "WARN",
                    Level::Info => "INFO",
                    Level::Debug => "DEBUG",
                    Level::Trace => "TRACE",
                };

                writeln!(
                    LOG_FILE.as_ref().unwrap(),
                    "[{:<5} {}:{}] {}",
                    level,
                    file_path,
                    record.line().unwrap_or(0),
                    record.args()
                )
                .unwrap();
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;
static mut LOG_FILE: Option<fs::File> = None;

pub fn init() -> Result<(), SetLoggerError> {
    // wipe old file
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("modloader_log.txt")
        .unwrap();

    // open file
    // unsafe because I'm too lazy to properly handle the file
    unsafe {
        LOG_FILE = Some(
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open("modloader_log.txt")
                .unwrap(),
        );
    }

    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}

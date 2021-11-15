//! Simple logger module.

use log::{LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        eprintln!("{}", record.args());
    }

    fn flush(&self) {}
}

pub fn init(verbosity: u64) -> Result<(), SetLoggerError> {
    let max_level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(max_level))
}

use log::{Metadata, Record};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(log_level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_boxed_logger(Box::new(Logger)).map(|()| log::set_max_level(log_level))
}

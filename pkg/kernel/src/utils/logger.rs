use log::{Metadata, Record};


pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIXME: Configure the logger
    log::set_max_level(log::LevelFilter::Info);
    //let log_level = parse_log_level(config.log_level);
    //log::set_max_level(log_level);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        println!("{}",record.args());
    }

    fn flush(&self) {}
}

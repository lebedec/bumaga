use std::time::Instant;

use log::{set_boxed_logger, set_max_level, LevelFilter, Log, Metadata, Record};

struct BasicLogger {
    start: Instant,
}

impl BasicLogger {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Log for BasicLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let timestamp = Instant::now().duration_since(self.start).as_secs_f32();
        println!(
            "{:.4} {} [{}] {}",
            timestamp,
            record.level(),
            record.module_path().unwrap_or("unknown"),
            record.args()
        )
    }

    fn flush(&self) {}
}

pub fn setup_tests_logging() {
    let _ = set_boxed_logger(Box::new(BasicLogger::new()));
    set_max_level(LevelFilter::Debug);
}

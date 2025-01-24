use agb::mgba::{DebugLevel, Mgba};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

/// Initializes logging through the MGBA Emulator
pub fn init() -> Result<(), SetLoggerError> {
    // SAFETY: Should be fine
    unsafe { log::set_logger_racy(&LOGGER).map(|()| log::set_max_level_racy(LevelFilter::Info)) }
}

struct MgbaLogger;

impl log::Log for MgbaLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Some(mut mgba) = Mgba::new() {
                let level = match record.level() {
                    Level::Error => Some(DebugLevel::Error),
                    Level::Warn => Some(DebugLevel::Warning),
                    Level::Info => Some(DebugLevel::Info),
                    Level::Debug => Some(DebugLevel::Debug),
                    Level::Trace => None,
                };

                if let Some(level) = level {
                    let _ = mgba.print(*record.args(), level);
                }
            }
        }
    }

    fn flush(&self) {}
}

const LOGGER: MgbaLogger = MgbaLogger;

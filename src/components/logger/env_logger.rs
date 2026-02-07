use crate::traits::Logger;

pub struct EnvLogger;

impl EnvLogger {
    pub fn new() -> Self {
        env_logger::init();
        EnvLogger
    }
}

impl Logger for EnvLogger {
    fn error(&self, msg: &str) {
        log::error!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        log::warn!("{}", msg);
    }

    fn info(&self, msg: &str) {
        log::info!("{}", msg);
    }

    fn debug(&self, msg: &str) {
        log::debug!("{}", msg);
    }

    fn trace(&self, msg: &str) {
        log::trace!("{}", msg);
    }
}

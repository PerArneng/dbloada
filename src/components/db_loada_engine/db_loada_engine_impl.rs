use crate::traits::{DbLoadaEngine, Logger};

pub struct DbLoadaEngineImpl {
    logger: Box<dyn Logger>,
}

impl DbLoadaEngineImpl {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        DbLoadaEngineImpl { logger }
    }
}

impl DbLoadaEngine for DbLoadaEngineImpl {
    fn init(&self) {
        self.logger.info("hello");
    }
}

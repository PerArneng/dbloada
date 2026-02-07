use crate::components::logger::EnvLogger;
use crate::components::db_loada_engine::DbLoadaEngineImpl;
use crate::traits::{DbLoadaEngine, Logger};

pub struct ComponentAssembler;

impl ComponentAssembler {
    pub fn new() -> Self {
        ComponentAssembler
    }

    pub fn logger(&self) -> Box<dyn Logger> {
        Box::new(EnvLogger::new())
    }

    pub fn db_loada_engine(&self) -> Box<dyn DbLoadaEngine> {
        Box::new(DbLoadaEngineImpl::new(self.logger()))
    }
}

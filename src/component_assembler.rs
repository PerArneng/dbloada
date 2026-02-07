use crate::components::logger::EnvLogger;
use crate::components::db_loada_engine::DbLoadaEngineImpl;
use crate::components::init::InitImpl;
use crate::components::load::LoadImpl;
use crate::components::string_file::DiskStringFile;
use crate::components::db_loada_project_serialization::YamlDbLoadaProjectSerialization;
use crate::components::db_loada_project_io::YamlDbLoadaProjectIO;
use crate::traits::{
    DbLoadaEngine, DbLoadaProjectIO, DbLoadaProjectSerialization, Init, Load, Logger, StringFile,
};

pub struct ComponentAssembler;

impl ComponentAssembler {
    pub fn new() -> Self {
        ComponentAssembler
    }

    pub fn logger(&self) -> Box<dyn Logger> {
        Box::new(EnvLogger::new())
    }

    pub fn init(&self) -> Box<dyn Init> {
        Box::new(InitImpl::new(self.logger(), self.db_loada_project_io()))
    }

    pub fn load(&self) -> Box<dyn Load> {
        Box::new(LoadImpl::new(self.logger(), self.db_loada_project_io()))
    }

    pub fn db_loada_engine(&self) -> Box<dyn DbLoadaEngine> {
        Box::new(DbLoadaEngineImpl::new(self.logger(), self.init(), self.load()))
    }

    pub fn string_file(&self) -> Box<dyn StringFile> {
        Box::new(DiskStringFile::new(self.logger()))
    }

    pub fn db_loada_project_serialization(&self) -> Box<dyn DbLoadaProjectSerialization> {
        Box::new(YamlDbLoadaProjectSerialization::new(self.logger()))
    }

    pub fn db_loada_project_io(&self) -> Box<dyn DbLoadaProjectIO> {
        Box::new(YamlDbLoadaProjectIO::new(
            self.logger(),
            self.string_file(),
            self.db_loada_project_serialization(),
        ))
    }
}

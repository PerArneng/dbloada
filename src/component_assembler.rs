use crate::components::logger::EnvLogger;
use crate::components::engine::EngineImpl;
use crate::components::init::InitImpl;
use crate::components::load::LoadImpl;
use crate::components::file_system::DiskFileSystem;
use crate::components::project_serialization::YamlProjectSerialization;
use crate::components::project_io::YamlProjectIO;
use crate::traits::{
    Engine, ProjectIO, ProjectSerialization, Init, Load, Logger, FileSystem,
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
        Box::new(InitImpl::new(self.logger(), self.project_io(), self.file_system()))
    }

    pub fn load(&self) -> Box<dyn Load> {
        Box::new(LoadImpl::new(self.logger(), self.project_io()))
    }

    pub fn engine(&self) -> Box<dyn Engine> {
        Box::new(EngineImpl::new(self.logger(), self.init(), self.load()))
    }

    pub fn file_system(&self) -> Box<dyn FileSystem> {
        Box::new(DiskFileSystem::new(self.logger()))
    }

    pub fn project_serialization(&self) -> Box<dyn ProjectSerialization> {
        Box::new(YamlProjectSerialization::new(self.logger()))
    }

    pub fn project_io(&self) -> Box<dyn ProjectIO> {
        Box::new(YamlProjectIO::new(
            self.logger(),
            self.file_system(),
            self.project_serialization(),
        ))
    }
}

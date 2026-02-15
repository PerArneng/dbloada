use crate::components::logger::TokioLogger;
use crate::components::engine::EngineImpl;
use crate::components::init::InitImpl;
use crate::components::load::LoadImpl;
use crate::components::file_system::DiskFileSystem;
use crate::components::project_serialization::YamlProjectSerialization;
use crate::components::project_io::YamlProjectIO;
use crate::components::csv_parser::CsvParserImpl;
use crate::components::table_reader::CsvTableReader;
use crate::components::table_reader::CmdCsvTableReader;
use crate::traits::{
    Engine, ProjectIO, ProjectSerialization, Init, Load, Logger, FileSystem, CsvParser, TableReader,
};

pub struct ComponentAssembler;

impl ComponentAssembler {
    pub fn new() -> Self {
        ComponentAssembler
    }

    pub fn logger(&self) -> Box<dyn Logger> {
        Box::new(TokioLogger::new())
    }

    pub fn init(&self) -> Box<dyn Init> {
        Box::new(InitImpl::new(self.logger(), self.project_io(), self.file_system()))
    }

    pub fn load(&self) -> Box<dyn Load> {
        Box::new(LoadImpl::new(self.logger(), self.project_io()))
    }

    pub fn csv_parser(&self) -> Box<dyn CsvParser> {
        Box::new(CsvParserImpl::new(self.logger()))
    }

    pub fn table_readers(&self) -> Vec<Box<dyn TableReader>> {
        vec![
            Box::new(CsvTableReader::new(self.logger(), self.file_system(), self.csv_parser())),
            Box::new(CmdCsvTableReader::new(self.logger(), self.csv_parser())),
        ]
    }

    pub fn engine(&self) -> Box<dyn Engine> {
        Box::new(EngineImpl::new(self.logger(), self.init(), self.load(), self.table_readers()))
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

mod traits;
mod components;
mod component_assembler;

use std::path::PathBuf;
use std::process;
use clap::{Parser, Subcommand};
use component_assembler::ComponentAssembler;

#[derive(Parser)]
#[command(name = "dbloada")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new dbloada project in the given directory
    Init {
        /// Directory to initialize
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,

        /// Project name (must be a valid Kubernetes resource name). Defaults to the directory name.
        #[arg(short, long)]
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let assembler = ComponentAssembler::new();
    let engine = assembler.db_loada_engine();

    match cli.command {
        Commands::Init { dir, name } => {
            if let Err(e) = engine.init_project_dir(&dir, name.as_deref()) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    }
}

mod traits;
mod components;
mod component_assembler;

use std::path::PathBuf;
use std::process;
use clap::{Parser, Subcommand};
use component_assembler::ComponentAssembler;

#[derive(Parser)]
#[command(name = "dbloada", version = env!("CARGO_PKG_VERSION"))]
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
    /// Load a dbloada project from the given directory
    Load {
        /// Directory containing the dbloada.yaml project file
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let assembler = ComponentAssembler::new();
    let engine = assembler.engine();

    match cli.command {
        Commands::Init { dir, name } => {
            if let Err(e) = engine.init_project_dir(&dir, name.as_deref()).await {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        Commands::Load { dir } => {
            match engine.load_project(&dir).await {
                Ok(project) => println!("{:#?}", project),
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
    }
}

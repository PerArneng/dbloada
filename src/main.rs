mod traits;
mod models;
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

        /// Force initialization even if the directory is not empty
        #[arg(short, long)]
        force: bool,
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
        Commands::Init { dir, name, force } => {
            if let Err(e) = engine.init_project_dir(&dir, name.as_deref(), force).await {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        Commands::Load { dir } => {
            let loaded_project = match engine.load_project(&dir).await {
                Ok(loaded_project) => loaded_project,
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            };
            println!("{:#?}", loaded_project.project);
            for table in &loaded_project.tables {
                print!("{}", models::table_to_string(table));
            }
        }
    }
}

//! MDBook doc-graph CLI entry point.
//!
//! This binary is called by mdbook during the build process.

use clap::{Parser, Subcommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use std::io;
use std::process;

use mdbook_doc_graph::DocGraphPreprocessor;

#[derive(Parser)]
#[command(name = "mdbook-doc-graph")]
#[command(about = "MDBook preprocessor to build document relationship graphs")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if this preprocessor supports a renderer
    Supports {
        /// The renderer to check
        renderer: String,
    },
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Supports { renderer }) => {
            let preprocessor = DocGraphPreprocessor::new();
            if preprocessor.supports_renderer(&renderer) {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
        None => {
            // Run as preprocessor (stdin/stdout)
            if let Err(e) = run_preprocessor() {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn run_preprocessor() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let preprocessor = DocGraphPreprocessor::new();

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: mdbook version mismatch. Built against {} but running with {}",
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;

    Ok(())
}

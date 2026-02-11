use anyhow::Result;
use clap::{Parser, Subcommand};
use high_cut::{Config, Processor};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run highlight extraction
    Run {
        /// Input video file
        input: PathBuf,
        
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
        
        /// Configuration file (YAML)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { input, output, config } => {
            let config = if let Some(config_path) = config {
                let content = tokio::fs::read_to_string(config_path).await?;
                serde_yaml::from_str(&content)?
            } else {
                Config::default()
            };

            let processor = Processor::new(config);
            processor.run(&input, &output).await?;
        }
    }

    Ok(())
}

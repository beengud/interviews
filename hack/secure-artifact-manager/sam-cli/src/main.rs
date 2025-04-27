mod client;
mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sam", version, about = "Secure Artifact Manager CLI", long_about = "A CLI for securely uploading, scanning, downloading, and managing artifacts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage artifacts in the store
    Artifact {
        #[command(subcommand)]
        action: ArtifactAction,
    },
}

#[derive(Subcommand)]
enum ArtifactAction {
    /// Upload a new artifact
    Upload {
        /// Path to the artifact file
        path: String,
    },
    /// List all artifacts
    List,
    /// Download an artifact by ID
    Download {
        /// Artifact ID
        id: String,
        /// Output path
        output: String,
    },
    /// Delete an artifact by ID
    Delete {
        /// Artifact ID
        id: String,
    },
    /// Trigger a vulnerability scan on the artifact
    Scan {
        /// Artifact ID
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Artifact { action } => match action {
            ArtifactAction::Upload { path } => commands::upload::upload(&path).await,
            ArtifactAction::List => commands::list::list().await,
            ArtifactAction::Download { id, output } => commands::download::download(&id, &output).await,
            ArtifactAction::Delete { id } => commands::delete::delete(&id).await,
            ArtifactAction::Scan { id } => commands::scan::scan(&id).await,
        },
    }
}

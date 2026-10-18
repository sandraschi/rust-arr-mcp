//! L2 `rust-arr-mcp`: CLI-first binary + MCP stdio server over arr-core.
//! Phase 1 scaffold: `status --json`. Full `mcp` stdio + dual aliases land in Phase 3.

use arr_core::{dispatch_auto, Command};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rust-arr-mcp",
    version,
    about = "Arr stack MCP + CLI (ArtCraft)"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Service status as JSON.
    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::Status => match dispatch_auto(Command::Status).await {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
    }
}

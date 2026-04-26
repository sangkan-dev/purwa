//! Empu — Purwa CLI (full generators land in Sprint 8).

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "empu",
    version,
    about = "Empu — Purwa CLI (Artisan-equivalent)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List registered HTTP routes (full `empu` + app integration in Sprint 8)
    #[command(name = "route:list")]
    RouteList,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::RouteList) => {
            eprintln!("empu route:list — stub (Sprint 2).");
            eprintln!(
                "Routes are collected from your app binary via `inventory`. To print the table:"
            );
            eprintln!("  println!(\"{{}}\", purwa::format_route_table());");
            eprintln!("Sprint 8: this command will delegate to your built application.");
        }
        None => {
            eprintln!("empu — Purwa CLI. Usage: empu route:list (stub) — see TASK.md Sprint 8.");
        }
    }
}

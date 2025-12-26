use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cses")]
#[command(about = "CSES Problem Set Solutions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Introductory Problems
    Introductory {
        /// Problem name to run
        problem: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Introductory { problem } => match problem.as_str() {
            "grid_path_description" => {
                cses::testing::run_all_tests("introductory", "grid_path_description",
                    cses::introductory::grid_path_description::solve);
            }
            _ => {
                eprintln!("Unknown problem: {}", problem);
                eprintln!("Available problems:");
                eprintln!("  grid_path_description");
                std::process::exit(1);
            }
        },
    }
}

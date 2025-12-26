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
    /// Dynamic Programming Problems
    DynamicProgramming {
        /// Problem name to run
        problem: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Introductory { problem } => {
            cses::introductory::tasks().run(&problem);
        }
        Commands::DynamicProgramming { problem } => {
            cses::dynamic_programming::tasks().run(&problem);
        }
    }
}

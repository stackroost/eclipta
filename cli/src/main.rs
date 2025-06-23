use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use figlet_rs::FIGfont;
#[derive(Parser)]
#[command(name = "eclipta", version, about = "🩺 CLI for tracing Linux system calls using eBPF")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Welcome,
    TraceExec,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
          Commands::Welcome => {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("ECLIPTA").unwrap();
        let gradient = [196, 198, 201, 207, 213, 219, 81]; 

        for (i, line) in figure.to_string().lines().enumerate() {
            let color = gradient.get(i % gradient.len()).unwrap_or(&15); 
            println!("\x1b[38;5;{}m{}\x1b[0m", color, line);
        }
        println!("\x1b[1;36mself-hosted observability platform\x1b[0m\n");
    }
        Commands::TraceExec => {
            println!(" Coming soon: trace_execve program will load and trace exec calls.");
        }
    }

    Ok(())
}

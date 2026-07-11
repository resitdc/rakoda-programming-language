use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "rpl")]
#[command(about = "Interpreter Rakoda Programming Language (RPL)", long_about = None)]
#[command(version = runtime::version())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        file: PathBuf,
        #[arg(short, long)]
        watch: bool,
        #[arg(long)]
        interpreter: bool,
    },
    Repl,
    Serve {
        file: PathBuf,
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
    },
    Fmt {
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file, watch, interpreter } => {
            let use_vm = !*interpreter;
            if !*watch {
                match runtime::run_file(file, use_vm) {
                    Ok(success) => {
                        if !success {
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                use notify::{Watcher, RecursiveMode};
                use std::sync::mpsc::channel;
                use std::time::Duration;

                print!("{}[2J{}[1;1H", 27 as char, 27 as char); // Clear screen
                println!("\x1b[32m⏳ Memulai watch mode untuk {}...\x1b[0m", file.display());
                if let Err(e) = runtime::run_file(file, use_vm) {
                    eprintln!("{}", e);
                }
                println!("\n\x1b[32m👀 Menunggu perubahan file...\x1b[0m");

                let (tx, rx) = channel();
                let mut watcher = notify::recommended_watcher(tx)?;
                watcher.watch(file, RecursiveMode::NonRecursive)?;

                let mut last_run = std::time::Instant::now();

                for res in rx {
                    match res {
                        Ok(event) => {
                            if event.kind.is_modify() {
                                if last_run.elapsed() > Duration::from_millis(500) {
                                    last_run = std::time::Instant::now();
                                    print!("{}[2J{}[1;1H", 27 as char, 27 as char); // Clear screen
                                    println!("\x1b[32m🔄 File berubah, menjalankan ulang...\x1b[0m\n");
                                    if let Err(e) = runtime::run_file(file, use_vm) {
                                        eprintln!("{}", e);
                                    }
                                    println!("\n\x1b[32m👀 Menunggu perubahan file...\x1b[0m");
                                }
                            }
                        }
                        Err(e) => eprintln!("Watch error: {:?}", e),
                    }
                }
            }
        }
        Commands::Repl => {
            println!("Memulai sesi REPL RPL. Ketik 'berhenti' untuk keluar.");
        }
        Commands::Serve { file, port } => {
            runtime::serve(file.clone(), *port).await?;
        }
        Commands::Fmt { file } => {
            println!("Memformat file: {}", file.display());
        }
    }

    Ok(())
}

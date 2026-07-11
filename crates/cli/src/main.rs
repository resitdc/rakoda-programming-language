use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};
use lexer::Lexer;
use parser::Parser as IplParser;
use interpreter::Interpreter;

#[derive(Parser)]
#[command(name = "ipl")]
#[command(about = "Interpreter Indonesia Programming Language (IPL)", long_about = None)]
#[command(version = "0.1.0")]
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
        vm: bool,
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

fn run_file(file: &PathBuf, use_vm: bool) -> Result<bool> {
    let kode_sumber = fs::read_to_string(file)
        .with_context(|| format!("Gagal membaca file: {}", file.display()))?;

    let mut lexer = Lexer::new(&kode_sumber);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e.tampilkan(&kode_sumber));
            return Ok(false);
        }
    };

    let mut parser = IplParser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e.tampilkan(&kode_sumber));
            return Ok(false);
        }
    };

    let program = ast::optimizer::optimize_program(program);

    if use_vm {
        let compiler = vm::Compiler::new();
        match compiler.compile(program) {
            Ok(chunk) => {
                let mut machine = vm::VM::new();
                vm::stdlib::register_all(&mut machine);
                
                if let Err(e) = machine.execute(chunk) {
                    eprintln!("VM Error: {}", e);
                    return Ok(false);
                }
                return Ok(true);
            }
            Err(e) => {
                eprintln!("Compiler Error: {}", e);
                return Ok(false);
            }
        }
    }

    let mut interpreter = Interpreter::baru();
    match interpreter.eval_program(program) {
        Ok(hasil) => {
            if hasil != interpreter::objek::Objek::Kosong {
                println!("{}", hasil);
            }
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e.tampilkan(&kode_sumber));
            Ok(false)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file, watch, vm } => {
            if !*watch {
                let success = run_file(file, *vm)?;
                if !success {
                    std::process::exit(1);
                }
            } else {
                use notify::{Watcher, RecursiveMode};
                use std::sync::mpsc::channel;
                use std::time::Duration;

                print!("{}[2J{}[1;1H", 27 as char, 27 as char); // Clear screen
                println!("\x1b[32m⏳ Memulai watch mode untuk {}...\x1b[0m", file.display());
                let _ = run_file(file, *vm);
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
                                    let _ = run_file(file, *vm);
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
            println!("Memulai sesi REPL IPL. Ketik 'berhenti' untuk keluar.");
        }
        Commands::Serve { file, port } => {
            web::start_server(file.clone(), *port).await?;
        }
        Commands::Fmt { file } => {
            println!("Memformat file: {}", file.display());
        }
    }

    Ok(())
}

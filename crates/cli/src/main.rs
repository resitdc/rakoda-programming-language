use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod pkg;

#[derive(Parser)]
#[command(name = "rpl")]
#[command(about = "Interpreter Rakoda Programming Language (RPL)", long_about = None)]
#[command(disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    version: bool,
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
    },
    Fmt {
        file: PathBuf,
    },
    #[command(alias = "inisialisasi")]
    Init,
    Instal {
        paket: Option<String>,
    },
    Hapus {
        paket: String,
    },
    Kill {
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("Rakoda Programming Language\nV{}", runtime::version());
        return Ok(());
    }

    match &cli.command {
        Some(Commands::Run {
            file,
            watch,
            interpreter,
        }) => {
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
                use notify::{RecursiveMode, Watcher};
                use std::sync::mpsc::channel;
                use std::time::Duration;

                print!("{}[2J{}[1;1H", 27 as char, 27 as char); // Clear screen
                println!(
                    "\x1b[32m⏳ Memulai watch mode untuk {}...\x1b[0m",
                    file.display()
                );
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
                            if event.kind.is_modify()
                                && last_run.elapsed() > Duration::from_millis(500) {
                                    last_run = std::time::Instant::now();
                                    print!("{}[2J{}[1;1H", 27 as char, 27 as char); // Clear screen
                                    println!(
                                        "\x1b[32m🔄 File berubah, menjalankan ulang...\x1b[0m\n"
                                    );
                                    if let Err(e) = runtime::run_file(file, use_vm) {
                                        eprintln!("{}", e);
                                    }
                                    println!("\n\x1b[32m👀 Menunggu perubahan file...\x1b[0m");
                                }
                        }
                        Err(e) => eprintln!("Watch error: {:?}", e),
                    }
                }
            }
        }
        Some(Commands::Repl) => {
            println!("Memulai sesi REPL RPL. Ketik 'berhenti' untuk keluar.");
        }
        Some(Commands::Serve { file }) => {
            use notify::{RecursiveMode, Watcher};
            use std::sync::mpsc::channel;
            use std::time::Duration;

            print!("{}[2J{}[1;1H", 27 as char, 27 as char);
            println!(
                "\x1b[32m🚀 Memulai Server Mode (Live Reload) untuk {}...\x1b[0m",
                file.display()
            );

            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("run")
                .arg(file)
                .spawn()?;

            let (tx, rx) = channel();
            let mut watcher = notify::recommended_watcher(tx)?;
            watcher.watch(&std::env::current_dir()?, RecursiveMode::Recursive)?;

            let mut last_run = std::time::Instant::now();

            for res in rx {
                match res {
                    Ok(event) => {
                        if event.kind.is_modify() {
                            let should_restart = event.paths.iter().any(|path| {
                                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                                ext == "rpl" || ext == "html"
                            });

                            if should_restart && last_run.elapsed() > Duration::from_millis(500) {
                                last_run = std::time::Instant::now();

                                let _ = child.kill();
                                let _ = child.wait();

                                print!("{}[2J{}[1;1H", 27 as char, 27 as char);
                                println!(
                                    "\x1b[32m🔄 Perubahan terdeteksi! Merestart server...\x1b[0m\n"
                                );

                                child =
                                    std::process::Command::new(std::env::current_exe().unwrap())
                                        .arg("run")
                                        .arg(file)
                                        .spawn()?;
                            }
                        }
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            }
        }
        Some(Commands::Fmt { file }) => {
            println!("Memformat file: {}", file.display());
            println!("Format selesai (fitur masih dalam pengembangan).");
        }
        Some(Commands::Init) => {
            let cwd = std::env::current_dir()?;
            pkg::inisialisasi(&cwd)?;
        }
        Some(Commands::Instal { paket }) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = pkg::instal(paket.clone()).await {
                    eprintln!("Gagal instal: {}", e);
                }
            });
        }
        Some(Commands::Hapus { paket }) => {
            pkg::hapus(paket)?;
        }
        Some(Commands::Kill { port }) => {
            println!("Mencoba mematikan proses di port {}...", port);
            let check_output = std::process::Command::new("lsof")
                .arg("-i")
                .arg(format!(":{}", port))
                .arg("-t")
                .output();

            match check_output {
                Ok(output) if !output.stdout.is_empty() => {
                    let pids = String::from_utf8_lossy(&output.stdout);
                    let mut success = false;
                    for pid in pids.trim().split('\n') {
                        let pid = pid.trim();
                        if !pid.is_empty()
                            && let Ok(status) = std::process::Command::new("kill")
                                .arg("-9")
                                .arg(pid)
                                .status()
                                && status.success() {
                                    println!(
                                        "\x1b[32mBerhasil mematikan proses (PID: {}) di port {}.\x1b[0m",
                                        pid, port
                                    );
                                    success = true;
                                }
                    }
                    if !success {
                        println!("\x1b[31mGagal mematikan proses di port {}.\x1b[0m", port);
                    }
                }
                _ => {
                    println!(
                        "\x1b[33mTidak ada proses yang berjalan di port {}.\x1b[0m",
                        port
                    );
                }
            }
        }
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}

use anyhow::Result;
use std::path::PathBuf;

/// Jalankan perintah `rpl run <file>`
pub fn handle_run(file: &std::path::Path, watch: bool) -> Result<()> {
    if !watch {
        runtime::run_file(file).map_err(|e| {
            eprintln!("{}", e);
            anyhow::anyhow!("Program selesai dengan error")
        })?;
    } else {
        jalankan_watch(file, None)?;
    }
    Ok(())
}

/// Jalankan perintah `rpl serve <file>` — live reload server
pub fn handle_serve(file: &std::path::Path) -> Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    println!(
        "\x1b[32m🚀 Memulai Server Mode (Live Reload) untuk {}...\x1b[0m",
        file.display()
    );

    let mut child = std::process::Command::new(std::env::current_exe()?)
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
                        println!("\x1b[32m🔄 Perubahan terdeteksi! Merestart server...\x1b[0m\n");

                        child = std::process::Command::new(std::env::current_exe()?)
                            .arg("run")
                            .arg(file)
                            .spawn()?;
                    }
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

/// Jalankan perintah `rpl repl`
pub fn handle_repl() -> Result<()> {
    println!("Memulai sesi REPL RPL. Ketik 'berhenti' untuk keluar.");
    Ok(())
}

/// Jalankan perintah `rpl fmt <file>`
pub fn handle_fmt(file: &std::path::Path) -> Result<()> {
    println!("Memformat file: {}", file.display());
    println!("Format selesai (fitur masih dalam pengembangan).");
    Ok(())
}

/// Jalankan perintah `rpl init`
pub fn handle_init() -> Result<()> {
    let cwd = std::env::current_dir()?;
    crate::pkg::inisialisasi(&cwd)
}

/// Jalankan perintah `rpl instal [paket]`
pub async fn handle_instal(paket: Option<String>) -> Result<()> {
    crate::pkg::instal(paket).await
}

/// Jalankan perintah `rpl hapus <paket>`
pub fn handle_hapus(paket: &str) -> Result<()> {
    crate::pkg::hapus(paket)
}

/// Jalankan perintah `rpl kill <port>`
pub fn handle_kill(port: u16) -> Result<()> {
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
                    && status.success()
                {
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
    Ok(())
}

/// Jalankan perintah `rpl cek <file>` — syntax check tanpa eksekusi
pub fn handle_cek(file: &PathBuf) -> Result<()> {
    let kode_sumber = std::fs::read_to_string(file)
        .map_err(|_| anyhow::anyhow!("Gagal membaca file: {}", file.display()))?;

    let mut lexer = lexer::Lexer::new(&kode_sumber);
    let mut errors: Vec<String> = Vec::new();

    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            errors.push(e.tampilkan(&kode_sumber));
            Vec::new()
        }
    };

    if !tokens.is_empty() {
        let mut parser = parser::Parser::new(tokens);
        let program = parser.parse_program();

        if !program.errors.is_empty() {
            for e in &program.errors {
                errors.push(e.tampilkan(&kode_sumber));
            }
        }

        if errors.is_empty() {
            let mut checker = typechecker::TypeChecker::new();
            let result = checker.check(&program);

            if !result.errors.is_empty() {
                for e in &result.errors {
                    let msg = format!(
                        "\x1b[1;36m--> \x1b[0m\x1b[1m{}:{}:{}\x1b[0m\n  \x1b[1;33mtype error\x1b[0m: {}",
                        file.display(),
                        e.lokasi.baris,
                        e.lokasi.kolom,
                        e.pesan
                    );
                    let full = if let Some(ref saran) = e.saran {
                        format!("{}\n  \x1b[1;32m💡 bantuan:\x1b[0m {}", msg, saran)
                    } else {
                        msg
                    };
                    errors.push(full);
                }
            }
        }

        if errors.is_empty() && !program.statements.is_empty() {
            println!(
                "\x1b[1;32m✓ Aman:\x1b[0m {} tidak memiliki error. ({} deklarasi ditemukan)",
                file.display(),
                program.statements.len()
            );
            Ok(())
        } else {
            eprintln!("\x1b[1;31m✗ Ditemukan {} error:\x1b[0m", errors.len());
            for (i, e) in errors.iter().enumerate() {
                eprintln!("  {}", e);
                if i < errors.len() - 1 {
                    eprintln!();
                }
            }
            std::process::exit(1);
        }
    } else if !errors.is_empty() {
        eprintln!("\x1b[1;31m✗ Ditemukan {} error:\x1b[0m", errors.len());
        for (i, e) in errors.iter().enumerate() {
            eprintln!("  {}", e);
            if i < errors.len() - 1 {
                eprintln!();
            }
        }
        std::process::exit(1);
    } else {
        Ok(())
    }
}

/// Jalankan watch mode untuk file — otomatis reload saat file berubah
fn jalankan_watch(file: &std::path::Path, _port: Option<u16>) -> Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    println!(
        "\x1b[32m⏳ Memulai watch mode untuk {}...\x1b[0m",
        file.display()
    );
    if let Err(e) = runtime::run_file(file) {
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
                if event.kind.is_modify() && last_run.elapsed() > Duration::from_millis(500) {
                    last_run = std::time::Instant::now();
                    print!("{}[2J{}[1;1H", 27 as char, 27 as char);
                    println!("\x1b[32m🔄 File berubah, menjalankan ulang...\x1b[0m\n");
                    if let Err(e) = runtime::run_file(file) {
                        eprintln!("{}", e);
                    }
                    println!("\n\x1b[32m👀 Menunggu perubahan file...\x1b[0m");
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

pub fn handle_lsp() -> Result<()> {
    lsp::run_lsp().map_err(|e| anyhow::anyhow!("LSP error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::{Cli, Commands};

    /// Test CLI parsing: `rpl --version`
    #[test]
    fn test_cli_parse_version() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "--version"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.version);
    }

    /// Test CLI parsing: `rpl -v`
    #[test]
    fn test_cli_parse_version_short() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "-v"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.version);
    }

    /// Test CLI parsing: `rpl run file.rpl`
    #[test]
    fn test_cli_parse_run() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "run", "test.rpl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Run { file, watch }) => {
                assert_eq!(file, &PathBuf::from("test.rpl"));
                assert!(!watch);
            }
            _ => panic!("Expected run command"),
        }
    }

    /// Test CLI parsing: `rpl run --watch file.rpl`
    #[test]
    fn test_cli_parse_run_watch() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "run", "--watch", "test.rpl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Run { file, watch }) => {
                assert_eq!(file, &PathBuf::from("test.rpl"));
                assert!(*watch);
            }
            _ => panic!("Expected run command with watch"),
        }
    }

    /// Test CLI parsing: `rpl repl`
    #[test]
    fn test_cli_parse_repl() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "repl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Repl) => (),
            _ => panic!("Expected repl command"),
        }
    }

    /// Test CLI parsing: `rpl serve server.rpl`
    #[test]
    fn test_cli_parse_serve() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "serve", "server.rpl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Serve { file }) => {
                assert_eq!(file, &PathBuf::from("server.rpl"));
            }
            _ => panic!("Expected serve command"),
        }
    }

    /// Test CLI parsing: `rpl fmt file.rpl`
    #[test]
    fn test_cli_parse_fmt() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "fmt", "file.rpl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Fmt { file }) => {
                assert_eq!(file, &PathBuf::from("file.rpl"));
            }
            _ => panic!("Expected fmt command"),
        }
    }

    /// Test CLI parsing: `rpl init`
    #[test]
    fn test_cli_parse_init() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "init"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Init) => (),
            _ => panic!("Expected init command"),
        }
    }

    /// Test CLI parsing: `rpl inisialisasi` (alias)
    #[test]
    fn test_cli_parse_init_alias() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "inisialisasi"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Init) => (),
            _ => panic!("Expected init (inisialisasi) command"),
        }
    }

    /// Test CLI parsing: `rpl instal paket:url`
    #[test]
    fn test_cli_parse_instal() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "instal", "github:user/repo"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Instal { paket }) => {
                assert_eq!(paket.as_deref(), Some("github:user/repo"));
            }
            _ => panic!("Expected instal command"),
        }
    }

    /// Test CLI parsing: `rpl instal` (no package — read from rpl.json)
    #[test]
    fn test_cli_parse_instal_no_package() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "instal"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Instal { paket }) => {
                assert!(paket.is_none());
            }
            _ => panic!("Expected instal command"),
        }
    }

    /// Test CLI parsing: `rpl hapus paket`
    #[test]
    fn test_cli_parse_hapus() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "hapus", "paket_saya"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Hapus { paket }) => {
                assert_eq!(paket, "paket_saya");
            }
            _ => panic!("Expected hapus command"),
        }
    }

    /// Test CLI parsing: `rpl kill 3000`
    #[test]
    fn test_cli_parse_kill() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "kill", "3000"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Kill { port }) => {
                assert_eq!(*port, 3000);
            }
            _ => panic!("Expected kill command"),
        }
    }

    /// Test CLI parsing: `rpl cek file.rpl`
    #[test]
    fn test_cli_parse_cek() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "cek", "file.rpl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Cek { file }) => {
                assert_eq!(file, &PathBuf::from("file.rpl"));
            }
            _ => panic!("Expected cek command"),
        }
    }

    /// Test CLI parsing: `rpl check file.rpl` (alias)
    #[test]
    fn test_cli_parse_check_alias() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "check", "file.rpl"]);
        assert!(cli.is_ok());
        match &cli.unwrap().command {
            Some(Commands::Cek { file }) => {
                assert_eq!(file, &PathBuf::from("file.rpl"));
            }
            _ => panic!("Expected check (cek) command"),
        }
    }

    /// Test CLI parsing: invalid command
    #[test]
    fn test_cli_parse_invalid_command() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "invalid"]);
        assert!(cli.is_err());
    }

    /// Test CLI parsing: `rpl run` without file (should fail)
    #[test]
    fn test_cli_parse_run_no_file() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl", "run"]);
        assert!(cli.is_err());
    }

    /// Test CLI: no subcommand shows help
    #[test]
    fn test_cli_no_command_shows_help() {
        use clap::Parser;
        let cli = Cli::try_parse_from(["rpl"]);
        assert!(cli.is_ok());
        assert!(cli.unwrap().command.is_none());
    }
}

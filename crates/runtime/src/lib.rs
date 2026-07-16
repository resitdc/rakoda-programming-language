use lexer::Lexer;
use parser::Parser as RplParser;
use std::fs;
use std::path::PathBuf;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn serve(file: PathBuf, port: u16) -> anyhow::Result<()> {
    web::start_server(file, port).await
}

/// Execute RPL code string and return captured stdout output.
/// Used for FFI bridge (RPL Studio). Does NOT print to console.
pub fn execute_string(kode_sumber: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(kode_sumber);
    let tokens = lexer
        .tokenize()
        .map_err(|e| e.tampilkan_dengan_file(kode_sumber, None))?;

    let mut parser = RplParser::new(tokens);
    let mut program = parser.parse_program();
    let errors = std::mem::take(&mut program.errors);
    if let Some(e) = errors.into_iter().next() {
        return Err(e.tampilkan_dengan_file(kode_sumber, None));
    }

    let program = ast::optimizer::optimize_program(program);

    let mut typechecker = typechecker::TypeChecker::new();
    let _ = typechecker.check(&program);

    let mut machine = vm::VM::new();
    vm::stdlib::register_all(&mut machine);
    machine.capture_output = true;
    machine.output_buffer.clear();

    let compiler = vm::Compiler::baru_dengan_base_path(&mut machine.heap, None);
    let chunk = compiler
        .compile(program)
        .map_err(|e| format!("Compiler Error: {}", e))?;

    match machine.execute(chunk) {
        Ok(()) => Ok(machine.output_buffer.clone()),
        Err((msg, opt_lokasi)) => {
            if let Some(lokasi) = opt_lokasi {
                let e = errors::RplError::Runtime { pesan: msg, lokasi };
                Err(e.tampilkan_dengan_file(kode_sumber, None))
            } else {
                Err(format!("VM Error: {}", msg))
            }
        }
    }
}

pub fn run_file(file: &std::path::Path) -> Result<(), String> {
    let kode_asli =
        fs::read_to_string(file).map_err(|_| format!("Gagal membaca file: {}", file.display()))?;

    let is_html_template = file.to_string_lossy().ends_with(".rpl.html");
    let kode_sumber = if is_html_template {
        stdlib::template::preprocess_template(&kode_asli)
    } else {
        kode_asli
    };

    let base_path = file.parent().map(|p| p.to_path_buf());
    let nama_file_buf = file.to_string_lossy().to_string();
    let nama_file = Some(nama_file_buf.as_str());
    run_source(&kode_sumber, base_path, nama_file)
}

pub fn run_source(
    kode_sumber: &str,
    base_path: Option<PathBuf>,
    nama_file: Option<&str>,
) -> Result<(), String> {
    let mut lexer = Lexer::new(kode_sumber);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return Err(e.tampilkan_dengan_file(kode_sumber, nama_file));
        }
    };

    let mut parser = RplParser::new(tokens);
    let mut program = parser.parse_program();
    let errors = std::mem::take(&mut program.errors);
    if let Some(e) = errors.into_iter().next() {
        return Err(e.tampilkan_dengan_file(kode_sumber, nama_file));
    }

    let program = ast::optimizer::optimize_program(program);

    // Type checking warnings
    let mut typechecker = typechecker::TypeChecker::new();
    let check_result = typechecker.check(&program);
    if !check_result.errors.is_empty() {
        if let Some(file) = nama_file {
            eprintln!(
                "\x1b[1;33m⚠️  Peringatan pengecekan tipe di {}:\x1b[0m",
                file
            );
        } else {
            eprintln!("\x1b[1;33m⚠️  Peringatan pengecekan tipe:\x1b[0m");
        }
        for e in &check_result.errors {
            if let Some(file) = nama_file {
                eprintln!(
                    "  \x1b[1;36m--> \x1b[0m{}:{}:{}: \x1b[1;33m{}\x1b[0m",
                    file, e.lokasi.baris, e.lokasi.kolom, e.pesan
                );
            } else {
                eprintln!(
                    "  \x1b[1;36m--> \x1b[0mbaris {}, kolom {}: \x1b[1;33m{}\x1b[0m",
                    e.lokasi.baris, e.lokasi.kolom, e.pesan
                );
            }
            if let Some(ref saran) = e.saran {
                eprintln!("  \x1b[1;32m💡 bantuan:\x1b[0m {}", saran);
            }
        }
        eprintln!();
    }

    let mut machine = vm::VM::new();
    vm::stdlib::register_all(&mut machine);
    // Set project_root agar fungsi bawaan VM bisa resolve path relatif
    // terhadap direktori file sumber, bukan CWD.
    machine.heap.project_root = base_path.clone();

    let compiler = vm::Compiler::baru_dengan_base_path(&mut machine.heap, base_path);
    match compiler.compile(program) {
        Ok(chunk) => {
            if let Err((msg, opt_lokasi)) = machine.execute(chunk) {
                if let Some(lokasi) = opt_lokasi {
                    let e = errors::RplError::Runtime { pesan: msg, lokasi };
                    return Err(e.tampilkan_dengan_file(kode_sumber, nama_file));
                } else {
                    return Err(format!("VM Error: {}", msg));
                }
            }
            Ok(())
        }
        Err(e) => Err(format!("Compiler Error: {}", e)),
    }
}

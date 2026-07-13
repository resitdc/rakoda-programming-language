use interpreter::Interpreter;
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

pub fn run_file(file: &PathBuf, use_vm: bool) -> Result<bool, String> {
    let kode_asli =
        fs::read_to_string(file).map_err(|_| format!("Gagal membaca file: {}", file.display()))?;

    let is_html_template = file.to_string_lossy().ends_with(".rpl.html");
    let kode_sumber = if is_html_template {
        interpreter::template::preprocess_template(&kode_asli)
    } else {
        kode_asli
    };

    let base_path = file.parent().map(|p| p.to_path_buf());
    run_source(&kode_sumber, use_vm, base_path)
}

pub fn run_source(
    kode_sumber: &str,
    use_vm: bool,
    base_path: Option<PathBuf>,
) -> Result<bool, String> {
    let mut lexer = Lexer::new(kode_sumber);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return Err(e.tampilkan(kode_sumber));
        }
    };

    let mut parser = RplParser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            return Err(e.tampilkan(kode_sumber));
        }
    };

    let program = ast::optimizer::optimize_program(program);

    if use_vm {
        let mut machine = vm::VM::new();
        vm::stdlib::register_all(&mut machine);

        let compiler = vm::Compiler::baru_dengan_base_path(&mut machine.heap, base_path);
        match compiler.compile(program) {
            Ok(chunk) => {
                if let Err((msg, opt_lokasi)) = machine.execute(chunk) {
                    if let Some(lokasi) = opt_lokasi {
                        let e = errors::RplError::Runtime { pesan: msg, lokasi };
                        return Err(e.tampilkan(kode_sumber));
                    } else {
                        return Err(format!("VM Error: {}", msg));
                    }
                }
                return Ok(true);
            }
            Err(e) => {
                return Err(format!("Compiler Error: {}", e));
            }
        }
    }

    let mut interpreter = Interpreter::baru();
    interpreter.base_path = base_path;
    match interpreter.eval_program(program) {
        Ok(hasil) => {
            if hasil != interpreter::objek::Objek::Kosong {
                println!("{}", hasil);
            }
            Ok(true)
        }
        Err(e) => Err(e.tampilkan(kode_sumber)),
    }
}

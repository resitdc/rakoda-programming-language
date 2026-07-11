use std::path::PathBuf;
use std::fs;
use lexer::Lexer;
use parser::Parser as RplParser;
use interpreter::Interpreter;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn serve(file: PathBuf, port: u16) -> anyhow::Result<()> {
    web::start_server(file, port).await
}

pub fn run_file(file: &PathBuf, use_vm: bool) -> Result<bool, String> {
    let kode_sumber = fs::read_to_string(file)
        .map_err(|_| format!("Gagal membaca file: {}", file.display()))?;

    run_source(&kode_sumber, use_vm)
}

pub fn run_source(kode_sumber: &str, use_vm: bool) -> Result<bool, String> {
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

        let compiler = vm::Compiler::new(&mut machine.heap);
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
    match interpreter.eval_program(program) {
        Ok(hasil) => {
            if hasil != interpreter::objek::Objek::Kosong {
                println!("{}", hasil);
            }
            Ok(true)
        }
        Err(e) => {
            Err(e.tampilkan(kode_sumber))
        }
    }
}

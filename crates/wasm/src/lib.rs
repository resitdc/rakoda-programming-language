use lexer::Lexer;
use parser::Parser as RplParser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn jalankan_kode(kode_sumber: &str) -> String {
    let mut lexer = Lexer::new(kode_sumber);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return e.tampilkan_dengan_file(kode_sumber, None),
    };

    let mut parser = RplParser::new(tokens);
    let mut program = parser.parse_program();
    let errors = std::mem::take(&mut program.errors);
    if let Some(e) = errors.into_iter().next() {
        return e.tampilkan_dengan_file(kode_sumber, None);
    }

    let program = ast::optimizer::optimize_program(program);

    let mut machine = vm::VM::new();
    vm::stdlib::register_all(&mut machine);
    machine.capture_output = true;
    machine.output_buffer.clear();

    let compiler = vm::Compiler::baru_dengan_base_path(&mut machine.heap, None);
    let chunk = match compiler.compile(program) {
        Ok(c) => c,
        Err(e) => return format!("Compiler Error: {}", e),
    };

    match machine.execute(chunk) {
        Ok(()) => machine.output_buffer.clone(),
        Err((msg, opt_lokasi)) => {
            if let Some(lokasi) = opt_lokasi {
                let e = errors::RplError::Runtime { pesan: msg, lokasi };
                e.tampilkan_dengan_file(kode_sumber, None)
            } else {
                format!("VM Error: {}", msg)
            }
        }
    }
}

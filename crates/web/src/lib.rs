use anyhow::Result;
use axum::{response::Html, routing::get, Router};
use lexer::Lexer;
use parser::Parser as RplParser;
use std::fs;
use std::path::PathBuf;
use tokio::net::TcpListener;

pub async fn start_server(file: PathBuf, mut port: u16) -> Result<()> {
    let app = Router::new().route(
        "/",
        get({
            let file_path = file.clone();
            move || async move { handle_request(file_path).await }
        }),
    );

    let listener = loop {
        let addr = format!("0.0.0.0:{}", port);
        match TcpListener::bind(&addr).await {
            Ok(l) => {
                println!(
                    "\x1b[32mMenjalankan server web RPL pada http://{}\x1b[0m",
                    addr
                );
                break l;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                let nano = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos();
                let mut next_port = 1024 + (nano % (65535 - 1024));
                next_port = next_port - (next_port % 13);
                if next_port < 1024 {
                    next_port += 13;
                }
                port = next_port as u16;
            }
            Err(e) => return Err(e.into()),
        }
    };

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_request(file: PathBuf) -> Html<String> {
    let kode_asli = match fs::read_to_string(&file) {
        Ok(k) => k,
        Err(_) => return Html("<h1>Error: Gagal membaca file script RPL.</h1>".to_string()),
    };

    let kode_sumber = if file.to_string_lossy().ends_with(".rpl.html") {
        stdlib::template::preprocess_template(&kode_asli)
    } else {
        kode_asli
    };

    let mut lexer = Lexer::new(&kode_sumber);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber))),
    };

    let mut parser = RplParser::new(tokens);
    let mut program = parser.parse_program();
    let errors = std::mem::take(&mut program.errors);
    if let Some(e) = errors.into_iter().next() {
        return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber)));
    }

    let program = ast::optimizer::optimize_program(program);

    let mut machine = vm::VM::new();
    machine.capture_output = true;
    vm::stdlib::register_all(&mut machine);

    let base_path = file.parent().map(|p| p.to_path_buf());
    let compiler = vm::Compiler::baru_dengan_base_path(&mut machine.heap, base_path);

    match compiler.compile(program) {
        Ok(chunk) => {
            if let Err((msg, opt_lokasi)) = machine.execute(chunk) {
                if let Some(lokasi) = opt_lokasi {
                    let e = errors::RplError::Runtime {
                        pesan: msg,
                        lokasi,
                    };
                    return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber)));
                } else {
                    return Html(format!("<pre>VM Error: {}</pre>", msg));
                }
            }
            Html(machine.output_buffer.clone())
        }
        Err(e) => Html(format!("<pre>Compiler Error: {}</pre>", e)),
    }
}
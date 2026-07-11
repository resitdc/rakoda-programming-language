use axum::{
    routing::get,
    Router,
    response::Html,
};
use tokio::net::TcpListener;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use lexer::Lexer;
use parser::Parser as RplParser;
use interpreter::Interpreter;

pub async fn start_server(file: PathBuf, mut port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get({
            let file_path = file.clone();
            move || async move {
                handle_request(file_path).await
            }
        }));

    let listener = loop {
        let addr = format!("0.0.0.0:{}", port);
        match TcpListener::bind(&addr).await {
            Ok(l) => {
                println!("Menjalankan server web RPL pada http://{}", addr);
                break l;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                let nano = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos();
                let mut next_port = 1024 + (nano % (65535 - 1024));
                next_port = next_port - (next_port % 13);
                if next_port < 1024 { next_port += 13; }
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
        interpreter::template::preprocess_template(&kode_asli)
    } else {
        kode_asli
    };

    let mut lexer = Lexer::new(&kode_sumber);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber))),
    };

    let mut parser = RplParser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber))),
    };

    let mut interpreter = Interpreter::baru_dengan_capture();
    interpreter.base_path = file.parent().map(|p| p.to_path_buf());
    if let Err(e) = interpreter.eval_program(program) {
        return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber)));
    }

    Html(interpreter.output_buffer)
}

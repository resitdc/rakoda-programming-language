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

pub async fn start_server(file: PathBuf, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get({
            let file_path = file.clone();
            move || async move {
                handle_request(file_path).await
            }
        }));

    let addr = format!("0.0.0.0:{}", port);
    println!("Menjalankan server web RPL pada http://{}", addr);
    let listener = TcpListener::bind(&addr).await?;
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
    if let Err(e) = interpreter.eval_program(program) {
        return Html(format!("<pre>{}</pre>", e.tampilkan(&kode_sumber)));
    }

    Html(interpreter.output_buffer)
}

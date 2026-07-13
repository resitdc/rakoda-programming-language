use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use crate::stdlib::http::request;
use crate::stdlib::http::response;
use ureq::unversioned::multipart::{Form, Part};

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    // http.upload(url, config)
    // config expects "file" key to contain file path
    env.borrow_mut().set(
        "upload".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() >= 2
                && let (Objek::String(url), Objek::Kamus(config_map)) = (&args[0], &args[1]) {
                    let start = Instant::now();
                    let (_, http_config) =
                        request::apply_config(ureq::post(url), &config_map.borrow());

                    // Extract file path
                    let file_path = if let Some(Objek::String(p)) = config_map.borrow().get("file")
                    {
                        p.clone()
                    } else {
                        "".to_string()
                    };

                    let agent = if let Some(t) = http_config.timeout {
                        ureq::Agent::config_builder()
                            .timeout_global(Some(t))
                            .build()
                            .into()
                    } else {
                        ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
                    };

                    let mut last_err: Result<http::Response<ureq::Body>, ureq::Error> =
                        Err(ureq::Error::StatusCode(0));
                    let max_attempts = http_config.max_retries + 1;

                    for attempt in 0..max_attempts {
                        let (req, _) = request::apply_config(agent.post(url), &config_map.borrow());

                        let result = match Part::file(&file_path) {
                            Ok(part) => {
                                let mut form = Form::new();
                                form = form.part("file", part);
                                req.send(form)
                            }
                            Err(_e) => Err(ureq::Error::StatusCode(0)), // Could not open file
                        };

                        match result {
                            Ok(resp) => {
                                let elapsed = start.elapsed();
                                return response::build_response_objek(resp, elapsed);
                            }
                            Err(e) => {
                                last_err = Err(e);
                                if attempt < max_attempts - 1 && http_config.retry_delay_ms > 0 {
                                    std::thread::sleep(std::time::Duration::from_millis(
                                        http_config.retry_delay_ms,
                                    ));
                                }
                            }
                        }
                    }

                    let elapsed = start.elapsed();
                    let mut respon_map = HashMap::new();
                    respon_map.insert("status".to_string(), Objek::Angka(0.0));
                    respon_map.insert(
                        "status_text".to_string(),
                        Objek::String(format!("NETWORK_ERROR: {:?}", last_err.unwrap_err())),
                    );
                    respon_map.insert("berhasil".to_string(), Objek::Boolean(false));
                    respon_map.insert(
                        "waktu".to_string(),
                        Objek::Angka(elapsed.as_millis() as f64),
                    );
                    respon_map.insert("ukuran".to_string(), Objek::Angka(0.0));
                    respon_map.insert("data".to_string(), Objek::Kosong);
                    respon_map.insert("body".to_string(), Objek::Kosong);
                    respon_map.insert(
                        "header".to_string(),
                        Objek::Kamus(Rc::new(RefCell::new(HashMap::new()))),
                    );
                    return Objek::Kamus(Rc::new(RefCell::new(respon_map)));
                }
            Objek::Kosong
        }),
    );
}

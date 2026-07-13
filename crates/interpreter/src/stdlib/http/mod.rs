pub mod client;
pub mod download;
pub mod request;
pub mod response;
pub mod upload;

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use crate::stdlib::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

fn handle_result(
    result: Result<http::Response<ureq::Body>, ureq::Error>,
    elapsed: std::time::Duration,
) -> Objek {
    match result {
        Ok(resp) => response::build_response_objek(resp, elapsed),
        Err(e) => {
            let mut respon_map = HashMap::new();
            respon_map.insert("status".to_string(), Objek::Angka(0.0));
            respon_map.insert(
                "status_text".to_string(),
                Objek::String(format!("NETWORK_ERROR: {:?}", e)),
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
            Objek::Kamus(Rc::new(RefCell::new(respon_map)))
        }
    }
}

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    // GET
    module_env.borrow_mut().set(
        "get".to_string(),
        Objek::FungsiBawaan(|args| {
            if !args.is_empty()
                && let Objek::String(url) = &args[0]
            {
                let start = Instant::now();
                let config = args.get(1).and_then(|c| {
                    if let Objek::Kamus(k) = c {
                        Some(k)
                    } else {
                        None
                    }
                });
                let config_ref = config.map(|c| unsafe { &*c.as_ptr() });

                let (_, http_config) =
                    request::apply_config(ureq::get(url), config_ref.unwrap_or(&HashMap::new()));
                let agent = if let Some(t) = http_config.timeout {
                    ureq::Agent::config_builder()
                        .timeout_global(Some(t))
                        .build()
                        .into()
                } else {
                    ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
                };

                let mut last_err = Err(ureq::Error::StatusCode(0));
                for attempt in 0..=http_config.max_retries {
                    let (req, _) = request::apply_config(
                        agent.get(url),
                        config_ref.unwrap_or(&HashMap::new()),
                    );
                    match req.call() {
                        Ok(res) => return handle_result(Ok(res), start.elapsed()),
                        Err(e) => {
                            last_err = Err(e);
                            if attempt < http_config.max_retries && http_config.retry_delay_ms > 0 {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    http_config.retry_delay_ms,
                                ));
                            }
                        }
                    }
                }
                return handle_result(last_err, start.elapsed());
            }
            Objek::Kosong
        }),
    );

    // DELETE
    module_env.borrow_mut().set(
        "delete".to_string(),
        Objek::FungsiBawaan(|args| {
            if !args.is_empty()
                && let Objek::String(url) = &args[0]
            {
                let start = Instant::now();
                let config = args.get(1).and_then(|c| {
                    if let Objek::Kamus(k) = c {
                        Some(k)
                    } else {
                        None
                    }
                });
                let config_ref = config.map(|c| unsafe { &*c.as_ptr() });

                let (_, http_config) =
                    request::apply_config(ureq::delete(url), config_ref.unwrap_or(&HashMap::new()));
                let agent = if let Some(t) = http_config.timeout {
                    ureq::Agent::config_builder()
                        .timeout_global(Some(t))
                        .build()
                        .into()
                } else {
                    ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
                };

                let mut last_err = Err(ureq::Error::StatusCode(0));
                for attempt in 0..=http_config.max_retries {
                    let (req, _) = request::apply_config(
                        agent.delete(url),
                        config_ref.unwrap_or(&HashMap::new()),
                    );
                    match req.call() {
                        Ok(res) => return handle_result(Ok(res), start.elapsed()),
                        Err(e) => {
                            last_err = Err(e);
                            if attempt < http_config.max_retries && http_config.retry_delay_ms > 0 {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    http_config.retry_delay_ms,
                                ));
                            }
                        }
                    }
                }
                return handle_result(last_err, start.elapsed());
            }
            Objek::Kosong
        }),
    );

    // POST
    module_env.borrow_mut().set(
        "post".to_string(),
        Objek::FungsiBawaan(|args| {
            if !args.is_empty()
                && let Objek::String(url) = &args[0]
            {
                let start = Instant::now();
                let body = args.get(1);
                let config = args.get(2).and_then(|c| {
                    if let Objek::Kamus(k) = c {
                        Some(k)
                    } else {
                        None
                    }
                });
                let config_ref = config.map(|c| unsafe { &*c.as_ptr() });

                let (_, http_config) =
                    request::apply_config(ureq::post(url), config_ref.unwrap_or(&HashMap::new()));
                let mut has_ct = false;
                if let Some(c) = config_ref
                    && let Some(Objek::Kamus(headers_map)) = c.get("header")
                {
                    for k in headers_map.borrow().keys() {
                        if k.to_lowercase() == "content-type" {
                            has_ct = true;
                            break;
                        }
                    }
                }
                let agent = if let Some(t) = http_config.timeout {
                    ureq::Agent::config_builder()
                        .timeout_global(Some(t))
                        .build()
                        .into()
                } else {
                    ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
                };

                let mut last_err = Err(ureq::Error::StatusCode(0));
                for attempt in 0..=http_config.max_retries {
                    let (mut req, _) = request::apply_config(
                        agent.post(url),
                        config_ref.unwrap_or(&HashMap::new()),
                    );

                    let result = if let Some(body_arg) = body {
                        match body_arg {
                            Objek::Kamus(_) | Objek::Array(_) => {
                                if !has_ct {
                                    req = req.header("Content-Type", "application/json");
                                }
                                let json_value = json::to_json(body_arg);
                                req.send_json(&json_value)
                            }
                            Objek::String(s) => req.send(s.clone()),
                            _ => req.send_empty(),
                        }
                    } else {
                        req.send_empty()
                    };

                    match result {
                        Ok(res) => return handle_result(Ok(res), start.elapsed()),
                        Err(e) => {
                            last_err = Err(e);
                            if attempt < http_config.max_retries && http_config.retry_delay_ms > 0 {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    http_config.retry_delay_ms,
                                ));
                            }
                        }
                    }
                }
                return handle_result(last_err, start.elapsed());
            }
            Objek::Kosong
        }),
    );

    // PUT
    module_env.borrow_mut().set(
        "put".to_string(),
        Objek::FungsiBawaan(|args| {
            if !args.is_empty()
                && let Objek::String(url) = &args[0]
            {
                let start = Instant::now();
                let body = args.get(1);
                let config = args.get(2).and_then(|c| {
                    if let Objek::Kamus(k) = c {
                        Some(k)
                    } else {
                        None
                    }
                });
                let config_ref = config.map(|c| unsafe { &*c.as_ptr() });

                let (_, http_config) =
                    request::apply_config(ureq::put(url), config_ref.unwrap_or(&HashMap::new()));
                let mut has_ct = false;
                if let Some(c) = config_ref
                    && let Some(Objek::Kamus(headers_map)) = c.get("header")
                {
                    for k in headers_map.borrow().keys() {
                        if k.to_lowercase() == "content-type" {
                            has_ct = true;
                            break;
                        }
                    }
                }
                let agent = if let Some(t) = http_config.timeout {
                    ureq::Agent::config_builder()
                        .timeout_global(Some(t))
                        .build()
                        .into()
                } else {
                    ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
                };

                let mut last_err = Err(ureq::Error::StatusCode(0));
                for attempt in 0..=http_config.max_retries {
                    let (mut req, _) = request::apply_config(
                        agent.put(url),
                        config_ref.unwrap_or(&HashMap::new()),
                    );

                    let result = if let Some(body_arg) = body {
                        match body_arg {
                            Objek::Kamus(_) | Objek::Array(_) => {
                                if !has_ct {
                                    req = req.header("Content-Type", "application/json");
                                }
                                let json_value = json::to_json(body_arg);
                                req.send_json(&json_value)
                            }
                            Objek::String(s) => req.send(s.clone()),
                            _ => req.send_empty(),
                        }
                    } else {
                        req.send_empty()
                    };

                    match result {
                        Ok(res) => return handle_result(Ok(res), start.elapsed()),
                        Err(e) => {
                            last_err = Err(e);
                            if attempt < http_config.max_retries && http_config.retry_delay_ms > 0 {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    http_config.retry_delay_ms,
                                ));
                            }
                        }
                    }
                }
                return handle_result(last_err, start.elapsed());
            }
            Objek::Kosong
        }),
    );

    // PATCH
    module_env.borrow_mut().set(
        "patch".to_string(),
        Objek::FungsiBawaan(|args| {
            if !args.is_empty()
                && let Objek::String(url) = &args[0]
            {
                let start = Instant::now();
                let body = args.get(1);
                let config = args.get(2).and_then(|c| {
                    if let Objek::Kamus(k) = c {
                        Some(k)
                    } else {
                        None
                    }
                });
                let config_ref = config.map(|c| unsafe { &*c.as_ptr() });

                let (_, http_config) =
                    request::apply_config(ureq::patch(url), config_ref.unwrap_or(&HashMap::new()));
                let mut has_ct = false;
                if let Some(c) = config_ref
                    && let Some(Objek::Kamus(headers_map)) = c.get("header")
                {
                    for k in headers_map.borrow().keys() {
                        if k.to_lowercase() == "content-type" {
                            has_ct = true;
                            break;
                        }
                    }
                }
                let agent = if let Some(t) = http_config.timeout {
                    ureq::Agent::config_builder()
                        .timeout_global(Some(t))
                        .build()
                        .into()
                } else {
                    ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
                };

                let mut last_err = Err(ureq::Error::StatusCode(0));
                for attempt in 0..=http_config.max_retries {
                    let (mut req, _) = request::apply_config(
                        agent.patch(url),
                        config_ref.unwrap_or(&HashMap::new()),
                    );

                    let result = if let Some(body_arg) = body {
                        match body_arg {
                            Objek::Kamus(_) | Objek::Array(_) => {
                                if !has_ct {
                                    req = req.header("Content-Type", "application/json");
                                }
                                let json_value = json::to_json(body_arg);
                                req.send_json(&json_value)
                            }
                            Objek::String(s) => req.send(s.clone()),
                            _ => req.send_empty(),
                        }
                    } else {
                        req.send_empty()
                    };

                    match result {
                        Ok(res) => return handle_result(Ok(res), start.elapsed()),
                        Err(e) => {
                            last_err = Err(e);
                            if attempt < http_config.max_retries && http_config.retry_delay_ms > 0 {
                                std::thread::sleep(std::time::Duration::from_millis(
                                    http_config.retry_delay_ms,
                                ));
                            }
                        }
                    }
                }
                return handle_result(last_err, start.elapsed());
            }
            Objek::Kosong
        }),
    );

    upload::register(&module_env);
    download::register(&module_env);
    client::register(&module_env);

    env.borrow_mut()
        .set("http".to_string(), Objek::Modul(module_env));
}

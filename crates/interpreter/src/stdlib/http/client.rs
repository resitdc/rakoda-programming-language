use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use super::request;
use super::response;
use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use crate::stdlib::json;

fn resolve_url(base_config: &HashMap<String, Objek>, url: &str) -> String {
    let base_url = base_config.get("base_url").and_then(|v| if let Objek::String(s) = v { Some(s.as_str()) } else { None });
    stdlib::http::resolve_url(base_url, url)
}

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    env.borrow_mut().set(
        "klien".to_string(),
        Objek::FungsiBawaan(|args| {
            let base_config = if !args.is_empty() {
                if let Objek::Kamus(k) = &args[0] {
                    k.borrow().clone()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };

            // Create Agent once for the connection pool!
            let (_, dummy_config) =
                request::apply_config(ureq::get("http://localhost"), &base_config);
            let agent: ureq::Agent = if let Some(t) = dummy_config.timeout {
                ureq::Agent::config_builder()
                    .timeout_global(Some(t))
                    .build()
                    .into()
            } else {
                ureq::Agent::new_with_config(ureq::Agent::config_builder().build())
            };

            let mut instance_map = HashMap::new();

            // GET
            let agent_get = agent.clone();
            let base_config_get = base_config.clone();
            instance_map.insert(
                "get".to_string(),
                Objek::MetodeBawaan(Rc::new(move |req_args| {
                    if !req_args.is_empty()
                        && let Objek::String(url) = &req_args[0]
                    {
                        let start = Instant::now();
                        let req_config = req_args
                            .get(1)
                            .and_then(|c| {
                                if let Objek::Kamus(k) = c {
                                    Some(k.borrow().clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let merged_config = request::merge_configs(&base_config_get, &req_config);
                        let final_url = resolve_url(&merged_config, url);

                        let (_, http_config) =
                            request::apply_config(ureq::get("http://localhost"), &merged_config);

                        let mut last_err: Result<http::Response<ureq::Body>, ureq::Error> =
                            Err(ureq::Error::StatusCode(0));
                        for attempt in 0..=http_config.max_retries {
                            let (req, _) =
                                request::apply_config(agent_get.get(&final_url), &merged_config);
                            match req.call() {
                                Ok(res) => {
                                    return response::build_response_objek(res, start.elapsed());
                                }
                                Err(e) => {
                                    last_err = Err(e);
                                    if attempt < http_config.max_retries
                                        && http_config.retry_delay_ms > 0
                                    {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            http_config.retry_delay_ms,
                                        ));
                                    }
                                }
                            }
                        }

                        let mut err_map = HashMap::new();
                        err_map.insert("status".to_string(), Objek::Angka(0.0));
                        err_map.insert(
                            "status_text".to_string(),
                            Objek::String(format!("NETWORK_ERROR: {:?}", last_err.unwrap_err())),
                        );
                        err_map.insert("berhasil".to_string(), Objek::Boolean(false));
                        err_map.insert(
                            "waktu".to_string(),
                            Objek::Angka(start.elapsed().as_millis() as f64),
                        );
                        return Objek::Kamus(Rc::new(RefCell::new(err_map)));
                    }
                    Objek::Kosong
                })),
            );

            // DELETE
            let agent_del = agent.clone();
            let base_config_del = base_config.clone();
            instance_map.insert(
                "delete".to_string(),
                Objek::MetodeBawaan(Rc::new(move |req_args| {
                    if !req_args.is_empty()
                        && let Objek::String(url) = &req_args[0]
                    {
                        let start = Instant::now();
                        let req_config = req_args
                            .get(1)
                            .and_then(|c| {
                                if let Objek::Kamus(k) = c {
                                    Some(k.borrow().clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let merged_config = request::merge_configs(&base_config_del, &req_config);
                        let final_url = resolve_url(&merged_config, url);

                        let (_, http_config) =
                            request::apply_config(ureq::get("http://localhost"), &merged_config);

                        let mut last_err: Result<http::Response<ureq::Body>, ureq::Error> =
                            Err(ureq::Error::StatusCode(0));
                        for attempt in 0..=http_config.max_retries {
                            let (req, _) =
                                request::apply_config(agent_del.delete(&final_url), &merged_config);
                            match req.call() {
                                Ok(res) => {
                                    return response::build_response_objek(res, start.elapsed());
                                }
                                Err(e) => {
                                    last_err = Err(e);
                                    if attempt < http_config.max_retries
                                        && http_config.retry_delay_ms > 0
                                    {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            http_config.retry_delay_ms,
                                        ));
                                    }
                                }
                            }
                        }
                        let mut err_map = HashMap::new();
                        err_map.insert("status".to_string(), Objek::Angka(0.0));
                        err_map.insert(
                            "status_text".to_string(),
                            Objek::String(format!("NETWORK_ERROR: {:?}", last_err.unwrap_err())),
                        );
                        err_map.insert("berhasil".to_string(), Objek::Boolean(false));
                        err_map.insert(
                            "waktu".to_string(),
                            Objek::Angka(start.elapsed().as_millis() as f64),
                        );
                        return Objek::Kamus(Rc::new(RefCell::new(err_map)));
                    }
                    Objek::Kosong
                })),
            );

            // POST
            let agent_post = agent.clone();
            let base_config_post = base_config.clone();
            instance_map.insert(
                "post".to_string(),
                Objek::MetodeBawaan(Rc::new(move |req_args| {
                    if !req_args.is_empty()
                        && let Objek::String(url) = &req_args[0]
                    {
                        let start = Instant::now();
                        let body = req_args.get(1);
                        let req_config = req_args
                            .get(2)
                            .and_then(|c| {
                                if let Objek::Kamus(k) = c {
                                    Some(k.borrow().clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let merged_config = request::merge_configs(&base_config_post, &req_config);
                        let final_url = resolve_url(&merged_config, url);

                        let (_, http_config) =
                            request::apply_config(ureq::get("http://localhost"), &merged_config);

                        let mut has_ct = false;
                        if let Some(Objek::Kamus(headers_map)) = merged_config.get("header") {
                            for k in headers_map.borrow().keys() {
                                if k.to_lowercase() == "content-type" {
                                    has_ct = true;
                                    break;
                                }
                            }
                        }

                        let mut last_err: Result<http::Response<ureq::Body>, ureq::Error> =
                            Err(ureq::Error::StatusCode(0));
                        for attempt in 0..=http_config.max_retries {
                            let (req, _) =
                                request::apply_config(agent_post.post(&final_url), &merged_config);

                            let result = if let Some(body_arg) = body {
                                match body_arg {
                                    Objek::Kamus(_) | Objek::Array(_) => {
                                        let mut req = req;
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
                                Ok(res) => {
                                    return response::build_response_objek(res, start.elapsed());
                                }
                                Err(e) => {
                                    last_err = Err(e);
                                    if attempt < http_config.max_retries
                                        && http_config.retry_delay_ms > 0
                                    {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            http_config.retry_delay_ms,
                                        ));
                                    }
                                }
                            }
                        }
                        let mut err_map = HashMap::new();
                        err_map.insert("status".to_string(), Objek::Angka(0.0));
                        err_map.insert(
                            "status_text".to_string(),
                            Objek::String(format!("NETWORK_ERROR: {:?}", last_err.unwrap_err())),
                        );
                        err_map.insert("berhasil".to_string(), Objek::Boolean(false));
                        err_map.insert(
                            "waktu".to_string(),
                            Objek::Angka(start.elapsed().as_millis() as f64),
                        );
                        return Objek::Kamus(Rc::new(RefCell::new(err_map)));
                    }
                    Objek::Kosong
                })),
            );

            // PUT
            let agent_put = agent.clone();
            let base_config_put = base_config.clone();
            instance_map.insert(
                "put".to_string(),
                Objek::MetodeBawaan(Rc::new(move |req_args| {
                    if !req_args.is_empty()
                        && let Objek::String(url) = &req_args[0]
                    {
                        let start = Instant::now();
                        let body = req_args.get(1);
                        let req_config = req_args
                            .get(2)
                            .and_then(|c| {
                                if let Objek::Kamus(k) = c {
                                    Some(k.borrow().clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let merged_config = request::merge_configs(&base_config_put, &req_config);
                        let final_url = resolve_url(&merged_config, url);

                        let (_, http_config) =
                            request::apply_config(ureq::get("http://localhost"), &merged_config);

                        let mut has_ct = false;
                        if let Some(Objek::Kamus(headers_map)) = merged_config.get("header") {
                            for k in headers_map.borrow().keys() {
                                if k.to_lowercase() == "content-type" {
                                    has_ct = true;
                                    break;
                                }
                            }
                        }

                        let mut last_err: Result<http::Response<ureq::Body>, ureq::Error> =
                            Err(ureq::Error::StatusCode(0));
                        for attempt in 0..=http_config.max_retries {
                            let (req, _) =
                                request::apply_config(agent_put.put(&final_url), &merged_config);

                            let result = if let Some(body_arg) = body {
                                match body_arg {
                                    Objek::Kamus(_) | Objek::Array(_) => {
                                        let mut req = req;
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
                                Ok(res) => {
                                    return response::build_response_objek(res, start.elapsed());
                                }
                                Err(e) => {
                                    last_err = Err(e);
                                    if attempt < http_config.max_retries
                                        && http_config.retry_delay_ms > 0
                                    {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            http_config.retry_delay_ms,
                                        ));
                                    }
                                }
                            }
                        }
                        let mut err_map = HashMap::new();
                        err_map.insert("status".to_string(), Objek::Angka(0.0));
                        err_map.insert(
                            "status_text".to_string(),
                            Objek::String(format!("NETWORK_ERROR: {:?}", last_err.unwrap_err())),
                        );
                        err_map.insert("berhasil".to_string(), Objek::Boolean(false));
                        err_map.insert(
                            "waktu".to_string(),
                            Objek::Angka(start.elapsed().as_millis() as f64),
                        );
                        return Objek::Kamus(Rc::new(RefCell::new(err_map)));
                    }
                    Objek::Kosong
                })),
            );

            // PATCH
            let agent_patch = agent.clone();
            let base_config_patch = base_config.clone();
            instance_map.insert(
                "patch".to_string(),
                Objek::MetodeBawaan(Rc::new(move |req_args| {
                    if !req_args.is_empty()
                        && let Objek::String(url) = &req_args[0]
                    {
                        let start = Instant::now();
                        let body = req_args.get(1);
                        let req_config = req_args
                            .get(2)
                            .and_then(|c| {
                                if let Objek::Kamus(k) = c {
                                    Some(k.borrow().clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let merged_config = request::merge_configs(&base_config_patch, &req_config);
                        let final_url = resolve_url(&merged_config, url);

                        let (_, http_config) =
                            request::apply_config(ureq::get("http://localhost"), &merged_config);

                        let mut has_ct = false;
                        if let Some(Objek::Kamus(headers_map)) = merged_config.get("header") {
                            for k in headers_map.borrow().keys() {
                                if k.to_lowercase() == "content-type" {
                                    has_ct = true;
                                    break;
                                }
                            }
                        }

                        let mut last_err: Result<http::Response<ureq::Body>, ureq::Error> =
                            Err(ureq::Error::StatusCode(0));
                        for attempt in 0..=http_config.max_retries {
                            let (req, _) = request::apply_config(
                                agent_patch.patch(&final_url),
                                &merged_config,
                            );

                            let result = if let Some(body_arg) = body {
                                match body_arg {
                                    Objek::Kamus(_) | Objek::Array(_) => {
                                        let mut req = req;
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
                                Ok(res) => {
                                    return response::build_response_objek(res, start.elapsed());
                                }
                                Err(e) => {
                                    last_err = Err(e);
                                    if attempt < http_config.max_retries
                                        && http_config.retry_delay_ms > 0
                                    {
                                        std::thread::sleep(std::time::Duration::from_millis(
                                            http_config.retry_delay_ms,
                                        ));
                                    }
                                }
                            }
                        }
                        let mut err_map = HashMap::new();
                        err_map.insert("status".to_string(), Objek::Angka(0.0));
                        err_map.insert(
                            "status_text".to_string(),
                            Objek::String(format!("NETWORK_ERROR: {:?}", last_err.unwrap_err())),
                        );
                        err_map.insert("berhasil".to_string(), Objek::Boolean(false));
                        err_map.insert(
                            "waktu".to_string(),
                            Objek::Angka(start.elapsed().as_millis() as f64),
                        );
                        return Objek::Kamus(Rc::new(RefCell::new(err_map)));
                    }
                    Objek::Kosong
                })),
            );

            Objek::Kamus(Rc::new(RefCell::new(instance_map)))
        }),
    );
}

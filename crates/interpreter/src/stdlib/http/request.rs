use crate::objek::Objek;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::collections::HashMap;
use std::time::Duration;

pub(crate) struct HttpConfig {
    pub timeout: Option<Duration>,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

pub(crate) fn apply_config<B>(
    mut req: ureq::RequestBuilder<B>,
    config: &HashMap<String, Objek>,
) -> (ureq::RequestBuilder<B>, HttpConfig) {
    let mut http_config = HttpConfig {
        timeout: None,
        max_retries: 0,
        retry_delay_ms: 0,
    };

    // Extract Timeout
    if let Some(Objek::Angka(t)) = config.get("timeout") {
        http_config.timeout = Some(Duration::from_millis(*t as u64));
    }

    // Extract Retry
    if let Some(retry_obj) = config.get("retry") {
        match retry_obj {
            Objek::Angka(n) => {
                http_config.max_retries = *n as u32;
            }
            Objek::Kamus(k) => {
                let map = k.borrow();
                if let Some(Objek::Angka(m)) = map.get("maksimal") {
                    http_config.max_retries = *m as u32;
                }
                if let Some(Objek::Angka(d)) = map.get("delay") {
                    http_config.retry_delay_ms = *d as u64;
                }
            }
            _ => {}
        }
    }

    // Apply Headers
    if let Some(Objek::Kamus(headers_map)) = config.get("header") {
        for (k, v) in headers_map.borrow().iter() {
            if let Objek::String(val_str) = v {
                req = req.header(k.as_str(), val_str.as_str());
            } else {
                req = req.header(k.as_str(), v.to_string().as_str());
            }
        }
    }

    // Apply Parameters (Query String)
    if let Some(Objek::Kamus(params_map)) = config.get("parameter") {
        for (k, v) in params_map.borrow().iter() {
            if let Objek::String(val_str) = v {
                req = req.query(k.as_str(), val_str.as_str());
            } else {
                req = req.query(k.as_str(), v.to_string().as_str());
            }
        }
    }

    // Apply Authentication
    if let Some(Objek::String(token)) = config.get("bearer") {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    if let Some(Objek::String(api_key)) = config.get("api_key") {
        req = req.header("x-api-key", api_key);
    }

    if let Some(Objek::Kamus(auth_map)) = config.get("auth") {
        let map = auth_map.borrow();
        if let (Some(Objek::String(user)), Some(Objek::String(pass))) =
            (map.get("username"), map.get("password"))
        {
            let combined = format!("{}:{}", user, pass);
            let encoded = STANDARD.encode(combined);
            req = req.header("Authorization", format!("Basic {}", encoded));
        }
    }

    (req, http_config)
}

pub(crate) fn merge_configs(
    base: &HashMap<String, Objek>,
    req_config: &HashMap<String, Objek>,
) -> HashMap<String, Objek> {
    let mut merged = base.clone();

    for (k, v) in req_config.iter() {
        if let Some(base_v) = merged.get_mut(k) {
            match (base_v, v) {
                (Objek::Kamus(base_map), Objek::Kamus(req_map)) => {
                    let mut new_inner = base_map.borrow().clone();
                    for (ik, iv) in req_map.borrow().iter() {
                        new_inner.insert(ik.clone(), iv.clone());
                    }
                    *merged.get_mut(k).unwrap() =
                        Objek::Kamus(std::rc::Rc::new(std::cell::RefCell::new(new_inner)));
                }
                _ => {
                    *merged.get_mut(k).unwrap() = v.clone();
                }
            }
        } else {
            merged.insert(k.clone(), v.clone());
        }
    }

    merged
}

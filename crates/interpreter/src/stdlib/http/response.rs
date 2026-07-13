use crate::objek::Objek;
use crate::stdlib::json; // We'll need to make json::from_json public
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

pub(crate) fn build_response_objek(
    mut response: http::Response<ureq::Body>,
    elapsed: Duration,
) -> Objek {
    let mut respon_map = HashMap::new();

    // Status
    let status_code = response.status().as_u16() as f64;
    respon_map.insert("status".to_string(), Objek::Angka(status_code));

    // Status Text
    let status_str = response.status().as_str().to_string();
    respon_map.insert("status_text".to_string(), Objek::String(status_str));

    // Berhasil (True if 2xx)
    let is_success = (200.0..300.0).contains(&status_code);
    respon_map.insert("berhasil".to_string(), Objek::Boolean(is_success));

    // Waktu tempuh (ms)
    respon_map.insert(
        "waktu".to_string(),
        Objek::Angka(elapsed.as_millis() as f64),
    );

    // Header
    let mut header_map = HashMap::new();
    for (k, v) in response.headers().iter() {
        if let Ok(v_str) = v.to_str() {
            header_map.insert(k.as_str().to_string(), Objek::String(v_str.to_string()));
        }
    }
    respon_map.insert(
        "header".to_string(),
        Objek::Kamus(Rc::new(RefCell::new(header_map))),
    );

    // Content-Type check
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let is_json = content_type.contains("application/json");

    // Body
    if let Ok(body_text) = response.body_mut().read_to_string() {
        respon_map.insert("ukuran".to_string(), Objek::Angka(body_text.len() as f64));
        respon_map.insert("body".to_string(), Objek::String(body_text.clone()));

        // Auto parse JSON if content-type is json
        if is_json {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body_text) {
                respon_map.insert("data".to_string(), json::from_json(&val));
            } else {
                // If parsing fails, fall back to string or Kosong
                respon_map.insert("data".to_string(), Objek::String(body_text));
            }
        } else {
            respon_map.insert("data".to_string(), Objek::String(body_text));
        }
    } else {
        respon_map.insert("ukuran".to_string(), Objek::Angka(0.0));
        respon_map.insert("body".to_string(), Objek::Kosong);
        respon_map.insert("data".to_string(), Objek::Kosong);
    }

    Objek::Kamus(Rc::new(RefCell::new(respon_map)))
}

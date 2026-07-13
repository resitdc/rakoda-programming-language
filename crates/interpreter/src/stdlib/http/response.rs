use crate::objek::Objek;
use crate::stdlib::json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

/// Build an Objek response from a ureq HTTP response.
///
/// Delegates the ureq-specific extraction to the shared stdlib
/// `extract_response_data`, then converts the shared data into
/// interpreter-native Objek values with JSON auto-parsing.
pub(crate) fn build_response_objek(
    response: http::Response<ureq::Body>,
    elapsed: Duration,
) -> Objek {
    let rd = stdlib::http::extract_response_data(response, elapsed);

    let mut respon_map = HashMap::new();

    respon_map.insert("status".to_string(), Objek::Angka(rd.status as f64));
    respon_map.insert("status_text".to_string(), Objek::String(rd.status_text));
    respon_map.insert("berhasil".to_string(), Objek::Boolean(rd.berhasil));
    respon_map.insert("waktu".to_string(), Objek::Angka(rd.waktu_ms as f64));

    // Headers
    let mut header_map = HashMap::new();
    for (k, v) in &rd.headers {
        header_map.insert(k.clone(), Objek::String(v.clone()));
    }
    respon_map.insert(
        "header".to_string(),
        Objek::Kamus(Rc::new(RefCell::new(header_map))),
    );

    respon_map.insert("ukuran".to_string(), Objek::Angka(rd.ukuran as f64));

    // Body & Data with JSON auto-parse
    if let Some(body) = &rd.data {
        respon_map.insert("body".to_string(), Objek::String(body.clone()));

        let content_type = rd
            .headers
            .get("content-type")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let is_json = content_type.contains("application/json");

        if is_json {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                respon_map.insert("data".to_string(), json::from_json(&val));
            } else {
                respon_map.insert("data".to_string(), Objek::String(body.clone()));
            }
        } else {
            respon_map.insert("data".to_string(), Objek::String(body.clone()));
        }
    } else {
        respon_map.insert("body".to_string(), Objek::Kosong);
        respon_map.insert("data".to_string(), Objek::Kosong);
    }

    Objek::Kamus(Rc::new(RefCell::new(respon_map)))
}
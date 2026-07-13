//! Tipe HTTP bersama untuk interpreter dan VM.
//!
//! Modul ini menyediakan tipe data netral (tidak terikat engine)
//! yang digunakan oleh baik interpreter maupun VM saat menangani
//! permintaan HTTP.

use std::collections::HashMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Konfigurasi
// ---------------------------------------------------------------------------

/// Konfigurasi untuk permintaan HTTP.
#[derive(Debug, Clone, Default)]
pub struct HttpConfig {
    pub timeout: Option<Duration>,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

// ---------------------------------------------------------------------------
// Data respons
// ---------------------------------------------------------------------------

/// Data respons HTTP yang diekstrak dari respons mentah (misalnya ureq).
/// Tipe ini netral dan dapat dikonversi ke Objek (interpreter) atau Value (VM).
#[derive(Debug, Clone, Default)]
pub struct HttpResponseData {
    pub status: u16,
    pub status_text: String,
    pub berhasil: bool,
    pub waktu_ms: u64,
    pub ukuran: usize,
    pub body: String,
    pub headers: std::collections::HashMap<String, String>,
    /// Data yang sudah diparse (JSON → string, atau body mentah).
    pub data: Option<String>,
}

// ---------------------------------------------------------------------------
// Konstanta kunci konfigurasi
// ---------------------------------------------------------------------------

pub const KEY_TIMEOUT: &str = "timeout";
pub const KEY_RETRY: &str = "retry";
pub const KEY_HEADER: &str = "header";
pub const KEY_PARAMETER: &str = "parameter";
pub const KEY_BEARER: &str = "bearer";
pub const KEY_API_KEY: &str = "api_key";
pub const KEY_AUTH: &str = "auth";

pub const KEY_RETRY_MAKSIMAL: &str = "maksimal";
pub const KEY_RETRY_DELAY: &str = "delay";

pub const KEY_USERNAME: &str = "username";
pub const KEY_PASSWORD: &str = "password";

/// Resolve URL relative to base URL.
///
/// If `url` already has a scheme (http:// or https://), return as-is.
/// Otherwise, join with `base_url` if provided, handling slash normalization.
pub fn resolve_url(base_url: Option<&str>, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }

    if let Some(base_url) = base_url {
        let mut final_url = base_url.to_string();
        if final_url.ends_with('/') && url.starts_with('/') {
            final_url.push_str(&url[1..]);
        } else if !final_url.ends_with('/') && !url.starts_with('/') {
            final_url.push('/');
            final_url.push_str(url);
        } else {
            final_url.push_str(url);
        }
        return final_url;
    }

    url.to_string()
}

/// Extract response data from a ureq HTTP response.
///
/// Reads status, headers, body, and timing information into a shared
/// `HttpResponseData` struct that both engines can consume.
pub fn extract_response_data(
    mut response: http::Response<ureq::Body>,
    elapsed: Duration,
) -> HttpResponseData {
    let status = response.status().as_u16();
    let status_text = response.status().as_str().to_string();
    let berhasil = (200..300).contains(&status);
    let waktu_ms = elapsed.as_millis() as u64;

    let mut headers = HashMap::new();
    for (k, v) in response.headers().iter() {
        if let Ok(v_str) = v.to_str() {
            headers.insert(k.as_str().to_string(), v_str.to_string());
        }
    }

    if let Ok(body_text) = response.body_mut().read_to_string() {
        let ukuran = body_text.len();
        HttpResponseData {
            status,
            status_text,
            berhasil,
            waktu_ms,
            ukuran,
            body: body_text.clone(),
            headers,
            data: Some(body_text),
        }
    } else {
        HttpResponseData {
            status,
            status_text,
            berhasil,
            waktu_ms,
            ukuran: 0,
            body: String::new(),
            headers,
            data: None,
        }
    }
}

/// Build an error response data struct (e.g., for network errors).
pub fn build_error_response_data(status_text: &str, elapsed: Duration) -> HttpResponseData {
    HttpResponseData {
        status: 0,
        status_text: status_text.to_string(),
        berhasil: false,
        waktu_ms: elapsed.as_millis() as u64,
        ukuran: 0,
        body: String::new(),
        headers: HashMap::new(),
        data: None,
    }
}

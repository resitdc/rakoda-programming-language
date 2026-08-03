use crate::jenis::NilaiRpl;
use lettre::message::{Attachment, MultiPart, SinglePart, header};
use lettre::{Message, SmtpTransport, Transport};
use std::collections::HashMap;
use std::fs;

// Karena `lettre` tidak diekspor untuk fungsi list statis biasa (hanya lewat VM konfigurasi),
// kita tidak memerlukan `fungsi_email()` yang standar, melainkan fungsi utilitas ini.
pub fn kirim_impl(
    config: &HashMap<String, NilaiRpl>,
    pesan: &HashMap<String, NilaiRpl>,
) -> Result<NilaiRpl, String> {
    // 1. Parsing Konfigurasi
    let host = get_str(config, "host").unwrap_or("localhost");
    let port = get_angka(config, "port").unwrap_or(25.0) as u16;
    let secure = get_bool(config, "secure").unwrap_or(false);

    let mut builder = if secure {
        let tls = lettre::transport::smtp::client::Tls::Wrapper(
            lettre::transport::smtp::client::TlsParameters::builder(host.to_string())
                .build()
                .map_err(|e| format!("Gagal konfigurasi TLS: {}", e))?,
        );
        SmtpTransport::builder_dangerous(host).port(port).tls(tls)
    } else if port == 587 {
        SmtpTransport::relay(host)
            .map_err(|e| format!("Gagal membuat SmtpTransport: {}", e))?
            .port(port)
    } else {
        SmtpTransport::builder_dangerous(host).port(port)
    };

    if let Some(NilaiRpl::Kamus(auth)) = config.get("auth") {
        let user = get_str(auth, "user").unwrap_or("");
        let pass = get_str(auth, "pass").unwrap_or("");
        if !user.is_empty() && !pass.is_empty() {
            let credentials = lettre::transport::smtp::authentication::Credentials::new(
                user.to_string(),
                pass.to_string(),
            );
            builder = builder.credentials(credentials);
        }
    }

    let mailer = builder.build();

    // 2. Parsing Pesan
    let mut msg_builder = Message::builder();

    if let Some(dari) = get_str(pesan, "dari") {
        msg_builder = msg_builder.from(dari.parse().map_err(|_| "Format 'dari' tidak valid")?);
    }
    if let Some(ke) = get_str(pesan, "ke") {
        for email in ke.split(',') {
            msg_builder = msg_builder.to(email
                .trim()
                .parse()
                .map_err(|_| format!("Format 'ke' tidak valid: {}", email))?);
        }
    } else if let Some(NilaiRpl::Daftar(ke_arr)) = pesan.get("ke") {
        for e in ke_arr {
            if let NilaiRpl::Teks(email) = e {
                msg_builder = msg_builder.to(email
                    .parse()
                    .map_err(|_| format!("Format 'ke' tidak valid: {}", email))?);
            }
        }
    }

    if let Some(NilaiRpl::Daftar(cc_arr)) = pesan.get("cc") {
        for e in cc_arr {
            if let NilaiRpl::Teks(email) = e {
                msg_builder = msg_builder.cc(email
                    .parse()
                    .map_err(|_| format!("Format 'cc' tidak valid: {}", email))?);
            }
        }
    }
    if let Some(bcc) = get_str(pesan, "bcc") {
        for email in bcc.split(',') {
            msg_builder = msg_builder.bcc(
                email
                    .trim()
                    .parse()
                    .map_err(|_| format!("Format 'bcc' tidak valid: {}", email))?,
            );
        }
    } else if let Some(NilaiRpl::Daftar(bcc_arr)) = pesan.get("bcc") {
        for e in bcc_arr {
            if let NilaiRpl::Teks(email) = e {
                msg_builder = msg_builder.bcc(
                    email
                        .parse()
                        .map_err(|_| format!("Format 'bcc' tidak valid: {}", email))?,
                );
            }
        }
    }

    if let Some(subjek) = get_str(pesan, "subjek") {
        msg_builder = msg_builder.subject(subjek);
    }

    let teks = get_str(pesan, "teks").unwrap_or("");
    let html = get_str(pesan, "html").unwrap_or("");

    let mut multipart = MultiPart::mixed().build();
    let mut has_content = false;

    if !html.is_empty() && !teks.is_empty() {
        multipart = multipart.multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(teks.to_string()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html.to_string()),
                ),
        );
        has_content = true;
    } else if !html.is_empty() {
        multipart = multipart.singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html.to_string()),
        );
        has_content = true;
    } else if !teks.is_empty() {
        multipart = multipart.singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(teks.to_string()),
        );
        has_content = true;
    }

    // Attachments
    if let Some(NilaiRpl::Daftar(lampiran)) = pesan.get("lampiran") {
        for l in lampiran {
            if let NilaiRpl::Kamus(k) = l {
                let nama_file = get_str(k, "nama_file").unwrap_or("lampiran");
                let path =
                    get_str(k, "path").ok_or_else(|| "Lampiran membutuhkan 'path'".to_string())?;
                let file_body = fs::read(path)
                    .map_err(|e| format!("Gagal membaca lampiran {}: {}", path, e))?;
                let content_type = header::ContentType::parse("application/octet-stream").unwrap();
                let attachment =
                    Attachment::new(nama_file.to_string()).body(file_body, content_type);
                multipart = multipart.singlepart(attachment);
                has_content = true;
            }
        }
    }

    if !has_content {
        return Err("Pesan harus memiliki 'teks', 'html', atau 'lampiran'".to_string());
    }

    let email = msg_builder
        .multipart(multipart)
        .map_err(|e| format!("Gagal membangun pesan: {}", e))?;

    // 3. Eksekusi Pengiriman
    match mailer.send(&email) {
        Ok(_) => Ok(NilaiRpl::Boolean(true)),
        Err(e) => Err(format!("Gagal mengirim email: {}", e)),
    }
}

// Helpers
fn get_str<'a>(k: &'a HashMap<String, NilaiRpl>, key: &str) -> Option<&'a str> {
    if let Some(NilaiRpl::Teks(s)) = k.get(key) {
        Some(s)
    } else {
        None
    }
}

fn get_angka(k: &HashMap<String, NilaiRpl>, key: &str) -> Option<f64> {
    if let Some(NilaiRpl::Angka(n)) = k.get(key) {
        Some(*n)
    } else {
        None
    }
}

fn get_bool(k: &HashMap<String, NilaiRpl>, key: &str) -> Option<bool> {
    if let Some(NilaiRpl::Boolean(b)) = k.get(key) {
        Some(*b)
    } else {
        None
    }
}

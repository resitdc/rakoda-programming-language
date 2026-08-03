use crate::{DaftarFungsiRpl, NilaiRpl};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value;

pub fn fungsi_jwt() -> DaftarFungsiRpl {
    vec![
        ("buat", buat_impl),
        ("verifikasi", verifikasi_impl),
        ("decode", decode_impl),
    ]
}

fn buat_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() != 2 {
        return Err("jwt.buat membutuhkan 2 argumen: payload, rahasia".to_string());
    }

    let payload_json = crate::json::ke_json(&args[0]);
    let rahasia = match &args[1] {
        NilaiRpl::Teks(t) => t,
        _ => return Err("Argumen rahasia harus berupa teks".to_string()),
    };

    let header = Header::new(Algorithm::HS256);
    let token = encode(
        &header,
        &payload_json,
        &EncodingKey::from_secret(rahasia.as_bytes()),
    )
    .map_err(|e| format!("Gagal membuat JWT: {}", e))?;

    Ok(NilaiRpl::Teks(token))
}

fn verifikasi_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() != 2 {
        return Err("jwt.verifikasi membutuhkan 2 argumen: token, rahasia".to_string());
    }

    let token = match &args[0] {
        NilaiRpl::Teks(t) => t,
        _ => return Err("Argumen token harus berupa teks".to_string()),
    };
    let rahasia = match &args[1] {
        NilaiRpl::Teks(t) => t,
        _ => return Err("Argumen rahasia harus berupa teks".to_string()),
    };

    let mut validation = Validation::new(Algorithm::HS256);
    // Nonaktifkan wajib punya exp untuk kemudahan, kecuali disetel oleh user
    validation.required_spec_claims.clear();
    validation.validate_exp = true; 

    let token_data = decode::<Value>(
        token,
        &DecodingKey::from_secret(rahasia.as_bytes()),
        &validation,
    )
    .map_err(|e| format!("Gagal verifikasi JWT: {}", e))?;

    Ok(crate::json::dari_json(&token_data.claims))
}

fn decode_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("jwt.decode membutuhkan argumen token".to_string());
    }

    let token = match &args[0] {
        NilaiRpl::Teks(t) => t,
        _ => return Err("Argumen token harus berupa teks".to_string()),
    };

    let token_data = jsonwebtoken::dangerous::insecure_decode::<Value>(token)
        .map_err(|e| format!("Gagal decode JWT: {}", e))?;

    Ok(crate::json::dari_json(&token_data.claims))
}

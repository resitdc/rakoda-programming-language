use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;
use sha2::{Digest, Sha256};

/// Fungsi-fungsi kripto murni (tidak tergantung engine).
/// Setiap fungsi menerima `&[NilaiRpl]` dan mengembalikan `Result<NilaiRpl, String>`.
pub fn fungsi_kripto() -> DaftarFungsiRpl {
    vec![("md5", md5_impl), ("sha256", sha256_impl)]
}

fn md5_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("kripto.md5 membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => {
            let digest = md5::compute(s.as_bytes());
            Ok(NilaiRpl::Teks(format!("{:x}", digest)))
        }
        _ => Err("kripto.md5 hanya menerima teks".to_string()),
    }
}

fn sha256_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("kripto.sha256 membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => {
            let mut hasher = Sha256::new();
            hasher.update(s.as_bytes());
            let result = hasher.finalize();
            Ok(NilaiRpl::Teks(format!("{:x}", result)))
        }
        _ => Err("kripto.sha256 hanya menerima teks".to_string()),
    }
}

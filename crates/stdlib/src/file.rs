use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;
use std::path::Path;

/// Fungsi-fungsi file murni (tidak tergantung engine).
pub fn fungsi_file() -> DaftarFungsiRpl {
    vec![
        ("tulis", tulis_impl),
        ("baca", baca_impl),
        ("ada", ada_impl),
        ("pindah", pindah_impl),
        ("hapus", hapus_impl),
    ]
}

fn tulis_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("file.tulis membutuhkan 2 argumen: nama_file, isi".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Teks(nama_file), NilaiRpl::Teks(isi)) => match std::fs::write(nama_file, isi) {
            Ok(_) => Ok(NilaiRpl::Boolean(true)),
            Err(e) => Err(format!("file.tulis gagal: {}", e)),
        },
        _ => Err("file.tulis membutuhkan argumen berupa teks".to_string()),
    }
}

fn baca_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("file.baca membutuhkan 1 argumen: nama_file".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(nama_file) => match std::fs::read_to_string(nama_file) {
            Ok(isi) => Ok(NilaiRpl::Teks(isi)),
            Err(e) => Err(format!("file.baca gagal: {}", e)),
        },
        _ => Err("file.baca hanya menerima teks".to_string()),
    }
}

fn ada_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("file.ada membutuhkan 1 argumen: nama_file".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(nama_file) => Ok(NilaiRpl::Boolean(Path::new(nama_file).exists())),
        _ => Err("file.ada hanya menerima teks".to_string()),
    }
}

fn pindah_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("file.pindah membutuhkan 2 argumen: asal, tujuan".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Teks(asal), NilaiRpl::Teks(tujuan)) => match std::fs::rename(asal, tujuan) {
            Ok(_) => Ok(NilaiRpl::Boolean(true)),
            Err(e) => Err(format!("file.pindah gagal: {}", e)),
        },
        _ => Err("file.pindah membutuhkan argumen berupa teks".to_string()),
    }
}

fn hapus_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("file.hapus membutuhkan 1 argumen: nama_file".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(nama_file) => match std::fs::remove_file(nama_file) {
            Ok(_) => Ok(NilaiRpl::Boolean(true)),
            Err(e) => Err(format!("file.hapus gagal: {}", e)),
        },
        _ => Err("file.hapus hanya menerima teks".to_string()),
    }
}

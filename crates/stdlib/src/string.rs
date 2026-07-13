use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;

/// Fungsi-fungsi string murni untuk manipulasi teks.
pub fn fungsi_string() -> DaftarFungsiRpl {
    vec![
        ("panjang", panjang_impl),
        ("besar", besar_impl),
        ("kecil", kecil_impl),
        ("potong", potong_impl),
        ("ganti", ganti_impl),
    ]
}

fn panjang_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.panjang membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Angka(s.len() as f64)),
        _ => Err("string.panjang hanya menerima teks".to_string()),
    }
}

fn besar_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.besar membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Teks(s.to_uppercase())),
        _ => Err("string.besar hanya menerima teks".to_string()),
    }
}

fn kecil_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.kecil membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Teks(s.to_lowercase())),
        _ => Err("string.kecil hanya menerima teks".to_string()),
    }
}

fn potong_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 3 {
        return Err("string.potong membutuhkan 3 argumen: teks, mulai, selesai".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (NilaiRpl::Teks(s), NilaiRpl::Angka(mulai), NilaiRpl::Angka(selesai)) => {
            let mulai = *mulai as usize;
            let selesai = *selesai as usize;
            if mulai > s.len() || selesai > s.len() || mulai > selesai {
                return Err("string.potong: indeks di luar jangkauan".to_string());
            }
            Ok(NilaiRpl::Teks(s[mulai..selesai].to_string()))
        }
        _ => Err("string.potong: argumen harus teks, angka, angka".to_string()),
    }
}

fn ganti_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 3 {
        return Err("string.ganti membutuhkan 3 argumen: teks, cari, ganti".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (NilaiRpl::Teks(s), NilaiRpl::Teks(cari), NilaiRpl::Teks(ganti)) => {
            Ok(NilaiRpl::Teks(s.replace(cari, ganti)))
        }
        _ => Err("string.ganti: argumen harus teks, teks, teks".to_string()),
    }
}

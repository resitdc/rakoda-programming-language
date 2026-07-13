use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;
use rand::Rng;

/// Fungsi-fungsi matematika murni untuk operasi numerik.
pub fn fungsi_matematika() -> DaftarFungsiRpl {
    vec![
        ("tambah", tambah_impl),
        ("kurang", kurang_impl),
        ("kali", kali_impl),
        ("bagi", bagi_impl),
        ("pangkat", pangkat_impl),
        ("bulatkan", bulatkan_impl),
        ("acak", acak_impl),
    ]
}

fn tambah_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("matematika.tambah membutuhkan 2 argumen angka".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Angka(a), NilaiRpl::Angka(b)) => Ok(NilaiRpl::Angka(a + b)),
        _ => Err("matematika.tambah hanya menerima angka".to_string()),
    }
}

fn kurang_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("matematika.kurang membutuhkan 2 argumen angka".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Angka(a), NilaiRpl::Angka(b)) => Ok(NilaiRpl::Angka(a - b)),
        _ => Err("matematika.kurang hanya menerima angka".to_string()),
    }
}

fn kali_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("matematika.kali membutuhkan 2 argumen angka".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Angka(a), NilaiRpl::Angka(b)) => Ok(NilaiRpl::Angka(a * b)),
        _ => Err("matematika.kali hanya menerima angka".to_string()),
    }
}

fn bagi_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("matematika.bagi membutuhkan 2 argumen angka".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Angka(a), NilaiRpl::Angka(b)) => {
            if *b == 0.0 {
                return Err("matematika.bagi tidak bisa membagi dengan nol".to_string());
            }
            Ok(NilaiRpl::Angka(a / b))
        }
        _ => Err("matematika.bagi hanya menerima angka".to_string()),
    }
}

fn pangkat_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("matematika.pangkat membutuhkan 2 argumen angka".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Angka(a), NilaiRpl::Angka(b)) => Ok(NilaiRpl::Angka(a.powf(*b))),
        _ => Err("matematika.pangkat hanya menerima angka".to_string()),
    }
}

fn bulatkan_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("matematika.bulatkan membutuhkan 1 argumen angka".to_string());
    }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.round())),
        _ => Err("matematika.bulatkan hanya menerima angka".to_string()),
    }
}

fn acak_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    let mut rng = rand::rng();
    if args.len() == 2
        && let (NilaiRpl::Angka(min), NilaiRpl::Angka(max)) = (&args[0], &args[1])
    {
        if max <= min {
            return Err("matematika.acak: max harus lebih besar dari min".to_string());
        }
        let hasil: f64 = rng.random_range(*min..*max);
        return Ok(NilaiRpl::Angka(hasil));
    }
    let hasil: f64 = rng.random_range(0.0..1.0);
    Ok(NilaiRpl::Angka(hasil))
}

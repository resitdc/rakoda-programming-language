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
        ("absolute", mutlak_impl),
        ("mutlak", mutlak_impl),
        ("sin", sin_impl),
        ("cos", cos_impl),
        ("tan", tan_impl),
        ("asin", asin_impl),
        ("acos", acos_impl),
        ("exp", exp_impl),
        ("log", log_impl),
        ("log10", log10_impl),
        ("sqrt", sqrt_impl),
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
    let mut rng = rand::thread_rng();
    if args.len() == 2
        && let (NilaiRpl::Angka(min), NilaiRpl::Angka(max)) = (&args[0], &args[1])
    {
        if max <= min {
            return Err("matematika.acak: max harus lebih besar dari min".to_string());
        }
        let hasil: f64 = rng.gen_range(*min..*max);
        return Ok(NilaiRpl::Angka(hasil));
    }
    let hasil: f64 = rng.gen_range(0.0..1.0);
    Ok(NilaiRpl::Angka(hasil))
}

fn mutlak_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("matematika.mutlak membutuhkan 1 argumen angka".to_string());
    }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.abs())),
        _ => Err("matematika.mutlak hanya menerima angka".to_string()),
    }
}

fn sin_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.sin membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.sin())),
        _ => Err("matematika.sin hanya menerima angka".to_string()),
    }
}

fn cos_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.cos membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.cos())),
        _ => Err("matematika.cos hanya menerima angka".to_string()),
    }
}

fn tan_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.tan membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.tan())),
        _ => Err("matematika.tan hanya menerima angka".to_string()),
    }
}

fn asin_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.asin membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.asin())),
        _ => Err("matematika.asin hanya menerima angka".to_string()),
    }
}

fn acos_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.acos membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.acos())),
        _ => Err("matematika.acos hanya menerima angka".to_string()),
    }
}

fn exp_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.exp membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.exp())),
        _ => Err("matematika.exp hanya menerima angka".to_string()),
    }
}

fn log_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.log membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.ln())), // ln() = natural log
        _ => Err("matematika.log hanya menerima angka".to_string()),
    }
}

fn log10_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.log10 membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.log10())),
        _ => Err("matematika.log10 hanya menerima angka".to_string()),
    }
}

fn sqrt_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() { return Err("matematika.sqrt membutuhkan 1 argumen angka".to_string()); }
    match &args[0] {
        NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(n.sqrt())),
        _ => Err("matematika.sqrt hanya menerima angka".to_string()),
    }
}

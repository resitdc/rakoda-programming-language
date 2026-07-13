use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;

/// Fungsi-fungsi core/global RPL (tampilkan, baca, angka, teks, boolean).
/// Ini adalah fungsi murni — tidak ada efek samping I/O.
/// Untuk I/O, engine yang akan menangani konversi NilaiRpl ke aksi nyata.
pub fn fungsi_core() -> DaftarFungsiRpl {
    vec![
        ("tampilkan", tampilkan_impl),
        ("baca", baca_impl),
        ("angka", angka_impl),
        ("teks", teks_impl),
        ("boolean", boolean_impl),
    ]
}

fn tampilkan_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    // Fungsi murni: gabungkan semua argumen jadi string
    let output = args
        .iter()
        .map(|a| match a {
            NilaiRpl::Teks(s) => s.clone(),
            NilaiRpl::Angka(n) => n.to_string(),
            NilaiRpl::Boolean(b) => b.to_string(),
            NilaiRpl::Kosong => "tidak ada".to_string(),
            _ => format!("{:?}", a),
        })
        .collect::<Vec<_>>()
        .join("");
    Ok(NilaiRpl::Teks(output))
}

fn baca_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    // Fungsi murni: kembalikan prompt. Engine yang akan handle I/O stdin.
    if let Some(NilaiRpl::Teks(p)) = args.first() {
        return Ok(NilaiRpl::Teks(p.clone()));
    }
    Ok(NilaiRpl::Teks(String::new()))
}

fn angka_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if let Some(arg) = args.first() {
        match arg {
            NilaiRpl::Angka(n) => Ok(NilaiRpl::Angka(*n)),
            NilaiRpl::Teks(s) => s
                .parse::<f64>()
                .map(NilaiRpl::Angka)
                .map_err(|_| format!("tidak bisa mengubah '{}' menjadi angka", s)),
            NilaiRpl::Boolean(b) => Ok(NilaiRpl::Angka(if *b { 1.0 } else { 0.0 })),
            _ => Ok(NilaiRpl::Kosong),
        }
    } else {
        Ok(NilaiRpl::Kosong)
    }
}

fn teks_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if let Some(arg) = args.first() {
        match arg {
            NilaiRpl::Teks(s) => Ok(NilaiRpl::Teks(s.clone())),
            NilaiRpl::Angka(n) => Ok(NilaiRpl::Teks(n.to_string())),
            NilaiRpl::Boolean(b) => Ok(NilaiRpl::Teks(b.to_string())),
            NilaiRpl::Kosong => Ok(NilaiRpl::Teks("tidak ada".to_string())),
            _ => Ok(NilaiRpl::Teks(format!("{:?}", arg))),
        }
    } else {
        Ok(NilaiRpl::Teks(String::new()))
    }
}

fn boolean_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if let Some(arg) = args.first() {
        match arg {
            NilaiRpl::Teks(s) => {
                let s_lower = s.to_lowercase();
                Ok(NilaiRpl::Boolean(
                    s_lower == "true" || s_lower == "benar" || s_lower == "1",
                ))
            }
            NilaiRpl::Angka(n) => Ok(NilaiRpl::Boolean(*n != 0.0)),
            NilaiRpl::Boolean(b) => Ok(NilaiRpl::Boolean(*b)),
            _ => Ok(NilaiRpl::Boolean(false)),
        }
    } else {
        Ok(NilaiRpl::Boolean(false))
    }
}

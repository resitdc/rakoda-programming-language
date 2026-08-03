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
        ("buat_folder", buat_folder_impl),
        ("daftar", daftar_impl),
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

fn buat_folder_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("file.buat_folder membutuhkan 1 argumen: path".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(path) => match std::fs::create_dir_all(path) {
            Ok(_) => Ok(NilaiRpl::Boolean(true)),
            Err(e) => Err(format!("file.buat_folder gagal: {}", e)),
        },
        _ => Err("file.buat_folder hanya menerima teks".to_string()),
    }
}

fn daftar_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err(
            "file.daftar membutuhkan 1 atau 2 argumen: path, tampilkan_hidden (opsional)"
                .to_string(),
        );
    }

    let path = match &args[0] {
        NilaiRpl::Teks(p) => p,
        _ => return Err("argumen pertama file.daftar harus teks (path)".to_string()),
    };

    let show_hidden = if args.len() > 1 {
        match &args[1] {
            NilaiRpl::Boolean(b) => *b,
            _ => return Err("argumen kedua file.daftar harus boolean".to_string()),
        }
    } else {
        false
    };

    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut result = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();

                    if !show_hidden && name_str.starts_with('.') {
                        continue;
                    }

                    result.push(NilaiRpl::Teks(name_str.to_string()));
                }
            }
            Ok(NilaiRpl::Daftar(result))
        }
        Err(e) => Err(format!("file.daftar gagal: {}", e)),
    }
}

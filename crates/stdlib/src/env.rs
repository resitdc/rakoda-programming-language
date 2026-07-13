use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;

/// Fungsi-fungsi environment murni (tidak tergantung engine).
pub fn fungsi_env() -> DaftarFungsiRpl {
    vec![("get", get_impl), ("set", set_impl), ("load", load_impl)]
}

fn get_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("env.get membutuhkan 1 argumen: nama_variabel".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(kunci) => match std::env::var(kunci) {
            Ok(val) => Ok(NilaiRpl::Teks(val)),
            Err(_) => Ok(NilaiRpl::Kosong),
        },
        _ => Err("env.get hanya menerima teks".to_string()),
    }
}

fn set_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("env.set membutuhkan 2 argumen: nama_variabel, nilai".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Teks(kunci), NilaiRpl::Teks(nilai)) => {
            // Safety: std::env::set_var aman dipanggil dari single-threaded RPL
            unsafe {
                std::env::set_var(kunci, nilai);
            }
            Ok(NilaiRpl::Boolean(true))
        }
        _ => Err("env.set membutuhkan argumen berupa teks".to_string()),
    }
}

fn load_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    let result = if args.is_empty() {
        dotenvy::dotenv()
    } else {
        match &args[0] {
            NilaiRpl::Teks(path) => dotenvy::from_filename(path),
            _ => return Err("env.load membutuhkan argumen berupa teks (path)".to_string()),
        }
    };
    match result {
        Ok(_) => Ok(NilaiRpl::Boolean(true)),
        Err(e) => Err(format!("env.load gagal: {}", e)),
    }
}

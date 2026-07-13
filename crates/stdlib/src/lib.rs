pub mod core;
pub mod env;
pub mod file;
pub mod http;
pub mod jenis;
pub mod json;
pub mod kripto;
pub mod list;
pub mod matematika;
pub mod string;
pub mod template;
pub mod waktu;

pub use jenis::NilaiRpl;

/// Tipe singkat untuk fungsi bawaan RPL.
pub type FungsiRpl = fn(&[NilaiRpl]) -> Result<NilaiRpl, String>;
/// Tipe singkat untuk daftar fungsi bawaan RPL.
pub type DaftarFungsiRpl = Vec<(&'static str, FungsiRpl)>;

/// Re-export semua fungsi dari masing-masing modul untuk kemudahan akses.
pub use core::fungsi_core;
pub use env::fungsi_env;
pub use file::fungsi_file;
pub use json::fungsi_json;
pub use kripto::fungsi_kripto;
pub use list::fungsi_list;
pub use matematika::fungsi_matematika;
pub use string::fungsi_string;
pub use waktu::fungsi_waktu;

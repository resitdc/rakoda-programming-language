/// Tipe nilai netral yang tidak terikat engine (interpreter maupun VM).
#[derive(Debug, Clone, PartialEq)]
pub enum NilaiRpl {
    Angka(f64),
    Teks(String),
    Boolean(bool),
    Kosong,
    Daftar(Vec<NilaiRpl>),
    Kamus(std::collections::HashMap<String, NilaiRpl>),
}

impl NilaiRpl {
    pub fn nama_tipe(&self) -> &str {
        match self {
            NilaiRpl::Angka(_) => "angka",
            NilaiRpl::Teks(_) => "teks",
            NilaiRpl::Boolean(_) => "boolean",
            NilaiRpl::Kosong => "kosong",
            NilaiRpl::Daftar(_) => "daftar",
            NilaiRpl::Kamus(_) => "kamus",
        }
    }
}

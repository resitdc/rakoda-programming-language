use thiserror::Error;

/// Posisi dalam source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub baris: usize,
    pub kolom: usize,
    pub offset: usize,
}

impl Pos {
    pub fn new(baris: usize, kolom: usize, offset: usize) -> Self {
        Self {
            baris,
            kolom,
            offset,
        }
    }
}

/// Rentang posisi dalam source code (start inclusive, end exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    /// Buat span dari posisi tunggal (untuk token yang hanya satu karakter)
    pub fn titik(start: Pos) -> Self {
        Self { start, end: start }
    }
}

/// Lokasi untuk backward compatibility. Akan dihapus setelah migrasi selesai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lokasi {
    pub baris: usize,
    pub kolom: usize,
}

impl Lokasi {
    pub fn new(baris: usize, kolom: usize) -> Self {
        Self { baris, kolom }
    }
}

impl From<Pos> for Lokasi {
    fn from(p: Pos) -> Self {
        Lokasi {
            baris: p.baris,
            kolom: p.kolom,
        }
    }
}

impl From<Span> for Lokasi {
    fn from(s: Span) -> Self {
        Lokasi {
            baris: s.start.baris,
            kolom: s.start.kolom,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum RplError {
    #[error("Sintaks tidak valid: {pesan}")]
    Sintaks {
        pesan: String,
        lokasi: Lokasi,
        saran: Option<String>,
    },

    #[error("Variabel '{nama}' belum dibuat.")]
    VariabelTidakDitemukan {
        nama: String,
        lokasi: Lokasi,
        saran: Option<String>,
    },

    #[error("Fungsi '{nama}' tidak ditemukan.")]
    FungsiTidakDitemukan {
        nama: String,
        lokasi: Lokasi,
        saran: Option<String>,
    },

    #[error("Tipe data tidak cocok: {pesan}")]
    TipeData {
        pesan: String,
        lokasi: Lokasi,
        saran: Option<String>,
    },

    #[error("Error internal: {pesan}")]
    Internal { pesan: String },

    #[error("Error runtime: {pesan}")]
    Runtime { pesan: String, lokasi: Lokasi },
}

impl RplError {
    pub fn tampilkan(&self, source_code: &str) -> String {
        self.tampilkan_dengan_file(source_code, None)
    }

    pub fn tampilkan_dengan_file(&self, source_code: &str, file: Option<&str>) -> String {
        match self {
            RplError::Sintaks {
                pesan,
                lokasi,
                saran,
            } => format_error(pesan, lokasi, saran, source_code, file),
            RplError::VariabelTidakDitemukan {
                nama,
                lokasi,
                saran,
            } => format_error(
                &format!("Variabel '{}' belum dibuat.", nama),
                lokasi,
                saran,
                source_code,
                file,
            ),
            RplError::FungsiTidakDitemukan {
                nama,
                lokasi,
                saran,
            } => format_error(
                &format!("Fungsi '{}' tidak ditemukan.", nama),
                lokasi,
                saran,
                source_code,
                file,
            ),
            RplError::TipeData {
                pesan,
                lokasi,
                saran,
            } => format_error(
                &format!("Tipe data tidak cocok: {}", pesan),
                lokasi,
                saran,
                source_code,
                file,
            ),
            RplError::Internal { pesan } => {
                format!("\x1b[33mKesalahan sistem internal: {}\x1b[0m", pesan)
            }
            RplError::Runtime { pesan, lokasi } => format_error(
                &format!("Runtime Error: {}", pesan),
                lokasi,
                &None,
                source_code,
                file,
            ),
        }
    }
}

fn format_error(
    pesan: &str,
    lokasi: &Lokasi,
    saran: &Option<String>,
    source_code: &str,
    file: Option<&str>,
) -> String {
    let baris_teks = source_code
        .lines()
        .nth(lokasi.baris.saturating_sub(1))
        .unwrap_or("");
    let pointer = " ".repeat(lokasi.kolom.saturating_sub(1)) + "^";

    let lokasi_str = if let Some(f) = file {
        format!("{}:{}:{}", f, lokasi.baris, lokasi.kolom)
    } else {
        format!("baris {}, kolom {}", lokasi.baris, lokasi.kolom)
    };

    let mut output = format!(
        "\x1b[33mError di {}:\n{}\n\n  {} | {}\n  {} | {}",
        lokasi_str,
        pesan,
        lokasi.baris,
        baris_teks,
        " ".repeat(lokasi.baris.to_string().len()),
        pointer
    );

    if let Some(s) = saran {
        output.push_str(&format!("\n\nSaran: {}", s));
    }

    output.push_str("\x1b[0m");

    output
}

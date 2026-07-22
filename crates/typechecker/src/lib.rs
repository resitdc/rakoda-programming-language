//! Type Checker untuk RPL (Rakoda Programming Language)
//!
//! Melakukan static type checking pada AST hasil parsing.
//! Masih tahap awal (Phase 3):

//! - Mendukung tipe primitif: Angka, String, Boolean, Kosong
//! - Array dan Kamus sederhana
//! - Fungsi dengan parameter
//! - Error-tolerant: mengumpulkan semua error, tidak bail cepat
//! - Pesan error dalam bahasa Indonesia

use ast::{Expression, InfixOperator, PrefixOperator, Program, Statement};
use errors::Lokasi;
use std::collections::HashMap;

// ============================================================================
// Tipe Representasi
// ============================================================================

/// Representasi tipe data dalam type system RPL.
#[derive(Debug, Clone, PartialEq)]
pub enum RplType {
    /// Tipe angka (bilangan pecahan)
    Angka,
    /// Tipe string (teks)
    String,
    /// Tipe boolean (benar/salah)
    Boolean,
    /// Tipe kosong (null/nil)
    Kosong,
    /// Tipe array (daftar)
    Array(Box<RplType>),
    /// Tipe kamus (key-value)
    Kamus(HashMap<String, RplType>),
    /// Tipe fungsi: parameter, return type
    Fungsi {
        params: Vec<(String, RplType)>,
        return_type: Box<RplType>,
    },
    /// Tipe tidak diketahui (inference gagal karena error sebelumnya)
    TidakDiketahui,
}

impl RplType {
    /// Nama tipe dalam bahasa Indonesia untuk pesan error.
    pub fn nama(&self) -> &'static str {
        match self {
            RplType::Angka => "angka",
            RplType::String => "teks",
            RplType::Boolean => "boolean",
            RplType::Kosong => "kosong",
            RplType::Array(_) => "daftar",
            RplType::Kamus(_) => "kamus",
            RplType::Fungsi { .. } => "fungsi",
            RplType::TidakDiketahui => "?",
        }
    }

    /// Cek apakah dua tipe kompatibel (bisa di-assign).
    pub fn kompatibel_dengan(&self, other: &RplType) -> bool {
        match (self, other) {
            (RplType::TidakDiketahui, _) | (_, RplType::TidakDiketahui) => true,
            (RplType::Kosong, _) | (_, RplType::Kosong) => true,
            (RplType::Angka, RplType::Angka) => true,
            (RplType::String, RplType::String) => true,
            (RplType::Boolean, RplType::Boolean) => true,
            (RplType::Array(a), RplType::Array(b)) => a.kompatibel_dengan(b),
            (
                RplType::Fungsi {
                    return_type: ra, ..
                },
                RplType::Fungsi {
                    return_type: rb, ..
                },
            ) => ra.kompatibel_dengan(rb),
            _ => false,
        }
    }

    /// Operator yang bisa digunakan pada tipe ini.
    pub fn operator_valid(&self, op: &InfixOperator) -> bool {
        matches!(
            (self, op),
            (RplType::Angka, _)
                | (RplType::String, InfixOperator::Tambah)
                | (RplType::String, InfixOperator::SamaDengan)
                | (RplType::String, InfixOperator::TidakSamaDengan)
                | (RplType::Boolean, InfixOperator::SamaDengan)
                | (RplType::Boolean, InfixOperator::TidakSamaDengan)
                | (RplType::Boolean, InfixOperator::Dan)
                | (RplType::Boolean, InfixOperator::Atau)
        )
    }
}

impl std::fmt::Display for RplType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RplType::Array(t) => write!(f, "daftar[{}]", t),
            RplType::Kamus(_) => write!(f, "kamus"),
            RplType::Fungsi {
                params,
                return_type,
            } => {
                let p: Vec<String> = params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect();
                write!(f, "fungsi({}) -> {}", p.join(", "), return_type)
            }
            RplType::TidakDiketahui => write!(f, "?"),
            _ => write!(f, "{}", self.nama()),
        }
    }
}

// ============================================================================
// Tabel Simbol
// ============================================================================

/// Satu entry dalam tabel simbol.
#[derive(Debug, Clone)]
struct Symbol {
    tipe: RplType,
    // Lokasi deklarasi disimpan untuk pelacakan error (belum digunakan)
    #[allow(dead_code)]
    dideklarasikan_di: Lokasi,
}

/// Tabel simbol dengan scope bersarang.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Masuk ke scope baru (misalnya masuk fungsi atau blok).
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Keluar dari scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Deklarasi variabel baru di scope saat ini.
    pub fn deklarasi(&mut self, nama: &str, tipe: RplType, lokasi: Lokasi) -> Result<(), String> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(nama) {
            return Err(format!(
                "Variabel '{}' sudah dideklarasikan di scope ini",
                nama
            ));
        }
        scope.insert(
            nama.to_string(),
            Symbol {
                tipe,
                dideklarasikan_di: lokasi,
            },
        );
        Ok(())
    }

    /// Cari tipe variabel di semua scope dari dalam ke luar.
    pub fn cari(&self, nama: &str) -> Option<&RplType> {
        // Scope lookup: cek semua scope dari terdalam ke terluar
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(nama) {
                return Some(&sym.tipe);
            }
        }
        None
    }

    /// Update tipe variabel yang sudah ada (untuk assignment).
    pub fn perbarui_tipe(&mut self, nama: &str, tipe_baru: RplType) -> Option<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.get_mut(nama) {
                sym.tipe = tipe_baru;
                return Some(());
            }
        }
        None
    }
}

// ============================================================================
// Type Checker
// ============================================================================

/// Error type checking.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub pesan: String,
    pub lokasi: Lokasi,
    pub saran: Option<String>,
}

/// Hasil utama type checker.
#[derive(Debug)]
pub struct CheckResult {
    /// Type error yang ditemukan.
    pub errors: Vec<TypeError>,
}

/// Type checker utama.
pub struct TypeChecker {
    symbols: SymbolTable,
    errors: Vec<TypeError>,
    /// Tipe return yang diharapkan saat ini (None = tidak ada ekspektasi)
    expected_return: Option<RplType>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut tc = Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            expected_return: None,
        };
        tc.daftarkan_builtin();
        tc
    }

    /// Daftarkan nama-nama built-in global yang tersedia di VM.
    /// Mencegah false positive "Variabel 'xxx' belum dibuat" pada built-in.
    pub fn daftarkan_builtin(&mut self) {
        let builtins = [
            // Global module/kamus yang di-register oleh vm::stdlib
            "db",
            "web",
            "string",
            "waktu",
            "kripto",
            "matematika",
            "list",
            "json",
            "http",
            "env",
            "file",
            "cookie",
            "session",
            "tugas",
            "log",
            "core",
            "ai",
            "regex",
            "uuid",
            "acak",
            "random",
            // Fungsi/konstanta global
            "panjang",
            "tipe",
            // Function bawaan dari core
            "tampilkan",
            "baca",
            "cetak",
            "tunda",
            // Http helpers
            "unduh",
            "kirim_http",
        ];

        let lokasi = errors::Lokasi::new(0, 0);
        for nama in &builtins {
            // Gunakan Kamus kosong sebagai representasi. Built-in ini
            // bisa berupa modul, kamus, atau fungsi bawaam. Yang penting
            // type checker tidak melaporkan "belum dibuat".
            let _ = self
                .symbols
                .deklarasi(nama, RplType::Kamus(HashMap::new()), lokasi);
        }
    }

    fn error(&mut self, pesan: String, lokasi: Lokasi, saran: Option<String>) {
        self.errors.push(TypeError {
            pesan,
            lokasi,
            saran,
        });
    }

    /// Entry point: check seluruh program.
    pub fn check(&mut self, program: &Program) -> CheckResult {
        for stmt in &program.statements {
            self.check_statement(stmt);
        }
        CheckResult {
            errors: std::mem::take(&mut self.errors),
        }
    }

    // ========================================================================
    // Statement checkers
    // ========================================================================

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::DeklarasiVariabel {
                nama,
                nilai,
                lokasi,
            } => {
                let tipe = self.infer_expression(nilai);
                // Jika nilai adalah kosong literal, biarkan tipe fleksibel
                let tipe_final = if let Expression::Kosong(_) = nilai {
                    RplType::Kosong
                } else {
                    tipe
                };
                if let Err(e) = self.symbols.deklarasi(nama, tipe_final.clone(), *lokasi) {
                    self.error(e, *lokasi, None);
                }
            }
            Statement::Assignment {
                nama,
                nilai,
                lokasi,
            } => {
                let tipe_nilai = self.infer_expression(nilai);
                match self.symbols.cari(nama) {
                    Some(tipe_var) => {
                        if !tipe_nilai.kompatibel_dengan(tipe_var) && tipe_var != &RplType::Kosong {
                            self.error(
                                format!(
                                    "Tidak bisa mengisi variabel '{}' bertipe '{}' dengan nilai bertipe '{}'",
                                    nama, tipe_var, tipe_nilai
                                ),
                                *lokasi,
                                Some(format!(
                                    "Pastikan nilai yang dimasukkan bertipe '{}', bukan '{}'",
                                    tipe_var, tipe_nilai
                                )),
                            );
                        }
                        self.symbols.perbarui_tipe(nama, tipe_nilai);
                    }
                    None => {
                        self.error(
                            format!("Variabel '{}' belum dibuat sebelum digunakan", nama),
                            *lokasi,
                            Some(format!(
                                "Buat variabel '{}' terlebih dahulu dengan 'buat {} = ...'",
                                nama, nama
                            )),
                        );
                    }
                }
            }
            Statement::DeklarasiFungsi {
                nama,
                parameter,
                body,
                lokasi,
            } => {
                // Infer tipe parameter dari pemanggilan (sementara: TidakDiketahui)
                // Full inference akan datang di phase selanjutnya
                let params: Vec<(String, RplType)> = parameter
                    .iter()
                    .map(|p| (p.clone(), RplType::TidakDiketahui))
                    .collect();
                let func_type = RplType::Fungsi {
                    params,
                    return_type: Box::new(RplType::Kosong),
                };
                if let Err(e) = self.symbols.deklarasi(nama, func_type, *lokasi) {
                    self.error(e, *lokasi, None);
                }

                // Check body fungsi dalam scope baru
                self.symbols.push_scope();
                // Deklarasi parameter di scope fungsi
                for p in parameter {
                    let _ = self.symbols.deklarasi(p, RplType::TidakDiketahui, *lokasi);
                }
                for s in body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
            }
            Statement::Kembalikan { nilai, lokasi } => {
                // Cek tipe return vs expected_return
                if let Some(val) = nilai {
                    let tipe = self.infer_expression(val);
                    if let Some(ref expected) = self.expected_return {
                        if !tipe.kompatibel_dengan(expected) {
                            self.error(
                                format!(
                                    "Fungsi seharusnya mengembalikan '{}', tapi mengembalikan '{}'",
                                    expected, tipe
                                ),
                                *lokasi,
                                Some("Periksa nilai yang dikembalikan".to_string()),
                            );
                        }
                    }
                }
            }
            Statement::Jika {
                kondisi,
                konsekuensi,
                alternatif,
                lokasi,
            } => {
                let tipe_kondisi = self.infer_expression(kondisi);
                if tipe_kondisi != RplType::Boolean && tipe_kondisi != RplType::TidakDiketahui {
                    self.error(
                        format!(
                            "Kondisi 'jika' harus berupa boolean (benar/salah), bukan '{}'",
                            tipe_kondisi
                        ),
                        *lokasi,
                        Some(
                            "Gunakan perbandingan seperti 'x > 5' atau 'nama == \"Budi\"'"
                                .to_string(),
                        ),
                    );
                }

                self.symbols.push_scope();
                for s in konsekuensi {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();

                if let Some(alt) = alternatif {
                    self.symbols.push_scope();
                    for s in alt {
                        self.check_statement(s);
                    }
                    self.symbols.pop_scope();
                }
            }
            Statement::Selama {
                kondisi,
                body,
                lokasi,
            } => {
                let tipe_kondisi = self.infer_expression(kondisi);
                if tipe_kondisi != RplType::Boolean && tipe_kondisi != RplType::TidakDiketahui {
                    self.error(
                        format!(
                            "Kondisi 'selama' harus berupa boolean, bukan '{}'",
                            tipe_kondisi
                        ),
                        *lokasi,
                        Some("Gunakan perbandingan seperti 'x > 0'".to_string()),
                    );
                }

                self.symbols.push_scope();
                for s in body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
            }
            Statement::Setiap {
                elemen,
                koleksi,
                indeks,
                body,
                lokasi,
            } => {
                let tipe_koleksi = self.infer_expression(koleksi);
                let tipe_elemen = match &tipe_koleksi {
                    RplType::Array(inner) => (**inner).clone(),
                    RplType::String => RplType::String,
                    RplType::Kamus(_) => RplType::TidakDiketahui, // Value from kamus
                    RplType::TidakDiketahui => RplType::TidakDiketahui,
                    _ => {
                        self.error(
                            format!(
                                "Hanya tipe daftar (array), teks (string), atau kamus yang bisa diulang. Diberikan '{}'",
                                tipe_koleksi
                            ),
                            *lokasi,
                            Some("Pastikan nilai yang diulang adalah daftar, teks, atau kamus.".to_string()),
                        );
                        RplType::TidakDiketahui
                    }
                };

                self.symbols.push_scope();
                let _ = self.symbols.deklarasi(elemen, tipe_elemen, *lokasi);
                
                if let Some(idx_name) = indeks {
                    let tipe_indeks = match tipe_koleksi {
                        RplType::Array(_) | RplType::String => RplType::Angka,
                        RplType::Kamus(_) => RplType::String,
                        _ => RplType::TidakDiketahui,
                    };
                    let _ = self.symbols.deklarasi(idx_name, tipe_indeks, *lokasi);
                }

                for s in body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
            }
            Statement::CobaTangkap {
                coba_body,
                error_ident,
                tangkap_body,
                lokasi,
            } => {
                self.symbols.push_scope();
                for s in coba_body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();

                self.symbols.push_scope();
                let _ = self
                    .symbols
                    .deklarasi(error_ident, RplType::String, *lokasi);
                for s in tangkap_body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
            }
            Statement::Tampilkan { nilai, .. } | Statement::Cetak { nilai, .. } => {
                // Tampilkan dan Cetak menerima semua tipe,
                // tapi tetap infer untuk deteksi variabel yang belum dibuat
                for expr in nilai {
                    let _ = self.infer_expression(expr);
                }
            }
            Statement::Lempar { nilai, lokasi: _ } => {
                let _ = self.infer_expression(nilai);
                // Lempar menerima semua tipe
            }
            Statement::Expression(expr) => {
                let _ = self.infer_expression(expr);
            }
            Statement::Error(_) => {
                // Error node dari parser, skip
            }
        }
    }

    // ========================================================================
    // Expression type inference
    // ========================================================================

    fn infer_expression(&mut self, expr: &Expression) -> RplType {
        match expr {
            Expression::Angka(_, _) => RplType::Angka,
            Expression::String(_, _) => RplType::String,
            Expression::Boolean(_, _) => RplType::Boolean,
            Expression::Kosong(_) => RplType::Kosong,
            Expression::Identifier(nama, lokasi) => match self.symbols.cari(nama) {
                Some(tipe) => tipe.clone(),
                None => {
                    self.error(
                        format!("Variabel '{}' belum dibuat", nama),
                        *lokasi,
                        Some(format!(
                            "Buat variabel '{}' terlebih dahulu dengan 'buat {} = ...'",
                            nama, nama
                        )),
                    );
                    RplType::TidakDiketahui
                }
            },
            Expression::Prefix {
                operator,
                kanan,
                lokasi,
            } => {
                let tipe_kanan = self.infer_expression(kanan);
                match operator {
                    PrefixOperator::Bukan => {
                        if tipe_kanan != RplType::Boolean && tipe_kanan != RplType::TidakDiketahui {
                            self.error(
                                format!("Operator 'bukan' hanya bisa digunakan untuk boolean, bukan '{}'", tipe_kanan),
                                *lokasi,
                                Some("Gunakan 'bukan' hanya untuk nilai benar/salah".to_string()),
                            );
                        }
                        RplType::Boolean
                    }
                    PrefixOperator::Minus => {
                        if tipe_kanan != RplType::Angka && tipe_kanan != RplType::TidakDiketahui {
                            self.error(
                                format!("Operator '-' (minus) hanya bisa digunakan untuk angka, bukan '{}'", tipe_kanan),
                                *lokasi,
                                Some("Gunakan minus hanya untuk angka".to_string()),
                            );
                        }
                        RplType::Angka
                    }
                }
            }
            Expression::Infix {
                kiri,
                operator,
                kanan,
                lokasi,
            } => {
                let tipe_kiri = self.infer_expression(kiri);
                let tipe_kanan = self.infer_expression(kanan);

                // Cek kompatibilitas operator
                if !tipe_kiri.operator_valid(operator) && tipe_kiri != RplType::TidakDiketahui {
                    self.error(
                        format!(
                            "Operator '{}' tidak bisa digunakan untuk tipe '{}'",
                            operator_ke_string(operator),
                            tipe_kiri
                        ),
                        *lokasi,
                        Some(format!("Operator '{}' hanya bisa digunakan untuk angka atau teks (tergantung operasi)", operator_ke_string(operator))),
                    );
                }

                // Tipe kiri dan kanan harus kompatibel
                // Pengecualian: operator + mendukung auto-coercion angka → teks
                let auto_coercion = matches!(operator, InfixOperator::Tambah)
                    && (tipe_kiri == RplType::String || tipe_kanan == RplType::String);
                if !auto_coercion
                    && tipe_kiri != RplType::TidakDiketahui
                    && tipe_kanan != RplType::TidakDiketahui
                    && tipe_kiri != tipe_kanan
                {
                    self.error(
                        format!(
                            "Tipe data tidak cocok: '{}' {} '{}'",
                            tipe_kiri,
                            operator_ke_string(operator),
                            tipe_kanan
                        ),
                        *lokasi,
                        Some(format!(
                            "Pastikan kedua sisi operator bertipe sama. Kiri: '{}', kanan: '{}'",
                            tipe_kiri, tipe_kanan
                        )),
                    );
                }

                // Tentukan tipe hasil
                match operator {
                    InfixOperator::Dan | InfixOperator::Atau => RplType::Boolean,
                    InfixOperator::SamaDengan
                    | InfixOperator::TidakSamaDengan
                    | InfixOperator::LebihDari
                    | InfixOperator::KurangDari
                    | InfixOperator::Minimal
                    | InfixOperator::Maksimal => RplType::Boolean,
                    InfixOperator::Tambah => {
                        if tipe_kiri == RplType::String || tipe_kanan == RplType::String {
                            RplType::String
                        } else {
                            RplType::Angka
                        }
                    }
                    InfixOperator::Kurang
                    | InfixOperator::Kali
                    | InfixOperator::Bagi
                    | InfixOperator::Mod => RplType::Angka,
                }
            }
            Expression::Call {
                fungsi,
                argumen,
                lokasi: _,
            } => {
                let tipe_fungsi = self.infer_expression(fungsi);
                // Check argumen
                for arg in argumen {
                    let _ = self.infer_expression(arg);
                }
                // Untuk fungsi bawaan, kita belum tahu return type-nya.
                // Kembalikan TidakDiketahui untuk sekarang.
                match tipe_fungsi {
                    RplType::Fungsi { return_type, .. } => (*return_type).clone(),
                    _ => RplType::TidakDiketahui,
                }
            }
            Expression::Array { elemen, lokasi: _ } => {
                let mut tipe_elemen: Option<RplType> = None;
                for el in elemen {
                    let t = self.infer_expression(el);
                    match &tipe_elemen {
                        None => tipe_elemen = Some(t),
                        Some(prev) if prev != &t && t != RplType::TidakDiketahui => {
                            // Heterogeneous array: izinkan, tapi catat
                            // Untuk sekarang, gunakan tipe elemen pertama
                        }
                        _ => {}
                    }
                }
                match tipe_elemen {
                    Some(t) => RplType::Array(Box::new(t)),
                    None => RplType::Array(Box::new(RplType::TidakDiketahui)),
                }
            }
            Expression::Kamus {
                pasangan,
                lokasi: _,
            } => {
                let mut map = HashMap::new();
                for (key, value) in pasangan {
                    let k =
                        match key {
                            Expression::String(s, _) => s.clone(),
                            Expression::Identifier(s, _) => s.clone(),
                            _ => {
                                self.error(
                                "Kunci kamus harus berupa teks".to_string(),
                                *key.lokasi(),
                                Some("Gunakan teks sebagai kunci, contoh: { \"nama\": \"Budi\" }"
                                    .to_string()),
                            );
                                continue;
                            }
                        };
                    let v = self.infer_expression(value);
                    map.insert(k, v);
                }
                RplType::Kamus(map)
            }
            Expression::Index {
                kiri,
                indeks,
                lokasi,
            } => {
                let tipe_kiri = self.infer_expression(kiri);
                let _tipe_indeks = self.infer_expression(indeks);

                match tipe_kiri {
                    RplType::Array(t) => (*t).clone(),
                    RplType::Kamus(_) => RplType::TidakDiketahui, // Nilai kamus bisa bertipe apa saja
                    RplType::String => RplType::String, // String indexing returns string? Seharusnya char, tapi RPL belum punya char
                    RplType::TidakDiketahui => RplType::TidakDiketahui,
                    _ => {
                        self.error(
                            format!("Tipe '{}' tidak bisa diakses dengan indeks [ ]", tipe_kiri),
                            *lokasi,
                            Some(
                                "Hanya daftar, kamus, dan teks yang bisa diakses dengan [ ]"
                                    .to_string(),
                            ),
                        );
                        RplType::TidakDiketahui
                    }
                }
            }
            Expression::FungsiAnonim {
                parameter,
                body,
                lokasi: _,
            } => {
                let params: Vec<(String, RplType)> = parameter
                    .iter()
                    .map(|p| (p.clone(), RplType::TidakDiketahui))
                    .collect();
                // Check body
                self.symbols.push_scope();
                for p in parameter {
                    let _ = self
                        .symbols
                        .deklarasi(p, RplType::TidakDiketahui, Lokasi::new(0, 0));
                }
                for s in body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();

                RplType::Fungsi {
                    params,
                    return_type: Box::new(RplType::Kosong),
                }
            }
            Expression::Impor(_, _lokasi) => {
                // Impor tidak bisa di-check secara static tanpa filesystem
                RplType::TidakDiketahui
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Konversi InfixOperator ke string yang mudah dibaca.
fn operator_ke_string(op: &InfixOperator) -> &'static str {
    match op {
        InfixOperator::Tambah => "+",
        InfixOperator::Kurang => "-",
        InfixOperator::Kali => "*",
        InfixOperator::Bagi => "/",
        InfixOperator::Mod => "%",
        InfixOperator::LebihDari => ">",
        InfixOperator::KurangDari => "<",
        InfixOperator::Minimal => ">=",
        InfixOperator::Maksimal => "<=",
        InfixOperator::SamaDengan => "==",
        InfixOperator::TidakSamaDengan => "!=",
        InfixOperator::Dan => "dan",
        InfixOperator::Atau => "atau",
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::Lexer;
    use parser::Parser;

    /// Helper: parse dan type-check input.
    fn check(input: &str) -> CheckResult {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();
        let mut checker = TypeChecker::new();
        checker.check(&program)
    }

    // ------------------------------------------------------------------------
    // Valid programs
    // ------------------------------------------------------------------------

    #[test]
    fn test_deklarasi_angka() {
        let result = check("buat x = 10");
        assert!(
            result.errors.is_empty(),
            "Seharusnya tidak ada error, tapi ada {} error: {:?}",
            result.errors.len(),
            result.errors
        );
    }

    #[test]
    fn test_deklarasi_string() {
        let result = check("buat nama = \"Budi\"");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_deklarasi_boolean() {
        let result = check("buat aktif = benar");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_operasi_aritmatika() {
        let result = check("buat total = 10 + 5 * 2");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_perbandingan() {
        let result = check("buat hasil = 10 > 5");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_jika_statement() {
        let result = check(
            "buat x = 10
jika x > 5 maka
    tampilkan \"besar\"
selesai",
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_fungsi_dasar() {
        let result = check(
            "fungsi sapa(nama)
    tampilkan nama
selesai",
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_array() {
        let result = check("buat daftar = [1, 2, 3]");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_kamus() {
        let result = check("buat data = { nama: \"Budi\", umur: 17 }");
        assert!(result.errors.is_empty());
    }

    // ------------------------------------------------------------------------
    // Type errors
    // ------------------------------------------------------------------------

    #[test]
    fn test_variabel_belum_dibuat() {
        let result = check("tampilkan x");
        assert!(!result.errors.is_empty());
        assert!(
            result.errors[0].pesan.contains("belum dibuat"),
            "Pesan error: {}",
            result.errors[0].pesan
        );
    }

    #[test]
    fn test_kondisi_jika_bukan_boolean() {
        let result = check(
            "buat x = 10
jika x maka
    tampilkan \"ya\"
selesai",
        );
        assert!(!result.errors.is_empty());
        assert!(
            result.errors[0].pesan.contains("boolean"),
            "Pesan error: {}",
            result.errors[0].pesan
        );
    }

    #[test]
    fn test_tipe_tidak_cocok_infix() {
        // Operator - tidak mendukung auto-coercion, jadi 10 - "halo" harus error
        let result = check("buat x = 10 - \"halo\"");
        assert!(!result.errors.is_empty());
        assert!(
            result.errors[0].pesan.contains("tidak cocok"),
            "Pesan error: {}",
            result.errors[0].pesan
        );
    }

    #[test]
    fn test_auto_coercion_angka_teks_diizinkan() {
        // Operator + antara string dan angka harus diizinkan (auto-coercion)
        let result = check("buat x = \"nilai: \" + 42");
        assert!(
            result.errors.is_empty(),
            "Auto-coercion angka ke teks seharusnya tidak error: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_operator_bukan_pada_angka() {
        let result = check("buat x = bukan 10");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_assignment_tipe_berbeda() {
        let result = check(
            "buat x = 10
x = \"halo\"",
        );
        // Assignment dengan tipe berbeda harus menghasilkan warning/error
        // Saat ini Kosong di-assign ke apa pun OK, tapi Angka ke String tidak
        assert!(!result.errors.is_empty());
    }

    // ------------------------------------------------------------------------
    // Error tolerance
    // ------------------------------------------------------------------------

    #[test]
    fn test_error_tidak_menghentikan_checking() {
        let result = check(
            "buat x = 10
tampilkan y
buat z = 20
tampilkan z",
        );
        // Harus ada error untuk 'y', tapi statement 'z' tetap di-check
        assert!(!result.errors.is_empty());
        // Cek bahwa error tentang 'y' bukan 'z'
        let y_error = result.errors.iter().any(|e| e.pesan.contains("y"));
        assert!(y_error, "Seharusnya ada error tentang 'y'");
    }

    #[test]
    fn test_banyak_error_dikumpulkan() {
        let result = check(
            "buat a = 10 - \"x\"
buat b = bukan 42
tampilkan c",
        );
        // Minimal 3 error
        assert!(
            result.errors.len() >= 3,
            "Seharusnya minimal 3 error, tapi dapat {}",
            result.errors.len()
        );
    }

    #[test]
    fn test_selama_kondisi_valid() {
        let result = check(
            "buat i = 0
selama i < 10 maka
    i = i + 1
selesai",
        );
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_coba_tangkap() {
        let result = check(
            "coba
    lempar \"error\"
tangkap (e)
    tampilkan e
selesai",
        );
        assert!(result.errors.is_empty());
    }
}

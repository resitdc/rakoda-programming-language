use errors::{Lokasi, RplError};
pub mod optimizer;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
    /// Error-error yang dikumpulkan selama parsing (non-fatal).
    /// Kosong = sukses penuh. Ada isi = tetap ada AST, tapi sebagian mungkin error node.
    pub errors: Vec<RplError>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    DeklarasiVariabel {
        nama: String,
        nilai: Expression,
        lokasi: Lokasi,
    },
    Jika {
        kondisi: Expression,
        konsekuensi: Vec<Statement>,
        alternatif: Option<Vec<Statement>>,
        lokasi: Lokasi,
    },
    Selama {
        kondisi: Expression,
        body: Vec<Statement>,
        lokasi: Lokasi,
    },
    Setiap {
        elemen: String,
        koleksi: Expression,
        indeks: Option<String>,
        body: Vec<Statement>,
        lokasi: Lokasi,
    },
    Kembalikan {
        nilai: Option<Expression>,
        lokasi: Lokasi,
    },
    DeklarasiFungsi {
        nama: String,
        parameter: Vec<String>,
        body: Vec<Statement>,
        lokasi: Lokasi,
    },
    Assignment {
        nama: String,
        nilai: Expression,
        lokasi: Lokasi,
    },
    Tampilkan {
        nilai: Vec<Expression>,
        lokasi: Lokasi,
    },
    Cetak {
        nilai: Vec<Expression>,
        lokasi: Lokasi,
    },
    CobaTangkap {
        coba_body: Vec<Statement>,
        error_ident: String,
        tangkap_body: Vec<Statement>,
        lokasi: Lokasi,
    },
    Lempar {
        nilai: Expression,
        lokasi: Lokasi,
    },
    Expression(Expression),
    /// Marker untuk potongan kode yang gagal diparse (error recovery).
    Error(Lokasi),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String, Lokasi),
    Angka(f64, Lokasi),
    String(String, Lokasi),
    Boolean(bool, Lokasi),
    Kosong(Lokasi),

    Impor(String, Lokasi),

    Prefix {
        operator: PrefixOperator,
        kanan: Box<Expression>,
        lokasi: Lokasi,
    },
    Infix {
        kiri: Box<Expression>,
        operator: InfixOperator,
        kanan: Box<Expression>,
        lokasi: Lokasi,
    },
    Call {
        fungsi: Box<Expression>,
        argumen: Vec<Expression>,
        lokasi: Lokasi,
    },
    Array {
        elemen: Vec<Expression>,
        lokasi: Lokasi,
    },
    Kamus {
        pasangan: Vec<(Expression, Expression)>,
        lokasi: Lokasi,
    },
    Index {
        kiri: Box<Expression>,
        indeks: Box<Expression>,
        lokasi: Lokasi,
    },
    FungsiAnonim {
        parameter: Vec<String>,
        body: Vec<Statement>,
        lokasi: Lokasi,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrefixOperator {
    Minus,
    Bukan,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InfixOperator {
    Tambah,
    Kurang,
    Kali,
    Bagi,
    Mod,
    LebihDari,
    KurangDari,
    Minimal,
    Maksimal,
    SamaDengan,
    TidakSamaDengan,
    Dan,
    Atau,
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements
    }
}

impl std::fmt::Display for InfixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            InfixOperator::Tambah => "+ (tambah)",
            InfixOperator::Kurang => "- (kurang)",
            InfixOperator::Kali => "* (kali)",
            InfixOperator::Bagi => "/ (bagi)",
            InfixOperator::Mod => "% (modulus)",
            InfixOperator::LebihDari => "> (lebih dari)",
            InfixOperator::KurangDari => "< (kurang dari)",
            InfixOperator::Minimal => ">= (minimal)",
            InfixOperator::Maksimal => "<= (maksimal)",
            InfixOperator::SamaDengan => "== (sama dengan)",
            InfixOperator::TidakSamaDengan => "!= (tidak sama dengan)",
            InfixOperator::Dan => "&& (dan)",
            InfixOperator::Atau => "|| (atau)",
        };
        write!(f, "{}", op)
    }
}
impl Expression {
    pub fn lokasi(&self) -> &Lokasi {
        match self {
            Expression::Identifier(_, l) => l,
            Expression::Angka(_, l) => l,
            Expression::String(_, l) => l,
            Expression::Boolean(_, l) => l,
            Expression::Kosong(l) => l,
            Expression::Prefix { lokasi, .. } => lokasi,
            Expression::Infix { lokasi, .. } => lokasi,
            Expression::Call { lokasi, .. } => lokasi,
            Expression::Impor(_, lokasi) => lokasi,

            Expression::Array { lokasi, .. } => lokasi,
            Expression::Kamus { lokasi, .. } => lokasi,
            Expression::Index { lokasi, .. } => lokasi,
            Expression::FungsiAnonim { lokasi, .. } => lokasi,
        }
    }
}

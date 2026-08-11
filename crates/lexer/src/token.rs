use errors::{Lokasi, Span};
// ini penting gaes, ini kamusnya

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Buat,       // buat
    Tetap,      // tetap
    Tampilkan,  // tampilkan
    Cetak,      // cetak
    Masukkan,   // masukkan
    Jika,       // jika
    JikaTidak,  // jika tidak
    AtauJika,   // atau jika
    Fungsi,     // fungsi
    Kembalikan, // kembalikan
    Selama,     // selama
    Ulangi,     // ulangi
    Setiap,     // setiap
    Di,         // di
    Dengan,     // dengan
    Indeks,     // indeks
    Berhenti,   // berhenti
    Lanjut,     // lanjut
    Impor,      // impor
    Benar,      // benar
    Salah,      // salah
    Kosong,     // kosong
    Maka,       // maka
    Selesai,    // selesai
    Coba,       // coba
    Tangkap,    // tangkap
    Lempar,     // lempar

    LebihDari,       // lebih dari
    KurangDari,      // kurang dari
    Minimal,         // minimal
    Maksimal,        // maksimal
    SamaDengan,      // sama dengan
    TidakSamaDengan, // tidak sama dengan
    Dan,             // dan
    Atau,            // atau
    Bukan,           // bukan

    Tambah,  // +
    Kurang,  // -
    Kali,    // *
    Bagi,    // /
    Mod,     // %
    Pangkat, // ^

    Assign,          // =
    TambahAssign,    // +=
    KurangAssign,    // -=
    KaliAssign,      // *=
    BagiAssign,      // /=
    ModAssign,       // %=
    Panah,           // =>
    TitikKoma,       // ;
    Koma,            // ,
    TitikDua,        // :
    Titik,           // .
    KurungBuka,      // (
    KurungTutup,     // )
    KurungSikuBuka,  // [
    KurungSikuTutup, // ]
    KurawalBuka,     // {
    KurawalTutup,    // }
    TandaTanya,      // ?
    TandaTanyaGanda, // ??
    BitwiseAtau,     // |

    Identifier(String),
    String(String),
    Angka(f64),
    AngkaImajiner(f64),

    EOF,
}

impl Token {
    pub fn to_indonesian_string(&self) -> String {
        match self {
            Token::Buat => "kata 'buat'".to_string(),
            Token::Tetap => "kata 'tetap'".to_string(),
            Token::Tampilkan => "kata 'tampilkan'".to_string(),
            Token::Cetak => "kata 'cetak'".to_string(),
            Token::Masukkan => "kata 'masukkan'".to_string(),
            Token::Jika => "kata 'jika'".to_string(),
            Token::JikaTidak => "kata 'jika tidak'".to_string(),
            Token::AtauJika => "kata 'atau jika'".to_string(),
            Token::Maka => "kata 'maka' atau '{'".to_string(),
            Token::Selama => "kata 'selama'".to_string(),
            Token::Setiap => "kata 'setiap'".to_string(),
            Token::Di => "kata 'di'".to_string(),
            Token::Dengan => "kata 'dengan'".to_string(),
            Token::Indeks => "kata 'indeks'".to_string(),
            Token::Fungsi => "kata 'fungsi'".to_string(),
            Token::Kembalikan => "kata 'kembalikan'".to_string(),
            Token::Impor => "kata 'impor' atau 'gabung'".to_string(),

            Token::Identifier(s) => format!("nama (variabel/fungsi) '{}'", s),
            Token::Angka(f) => format!("angka {}", f),
            Token::AngkaImajiner(f) => format!("angka kompleks {}j", f),
            Token::String(s) => format!("teks \"{}\"", s),

            Token::Assign => "tanda sama dengan '='".to_string(),
            Token::TambahAssign => "tanda tambah sama dengan '+='".to_string(),
            Token::KurangAssign => "tanda kurang sama dengan '-='".to_string(),
            Token::KaliAssign => "tanda '*='".to_string(),
            Token::BagiAssign => "tanda '/='".to_string(),
            Token::ModAssign => "tanda '%='".to_string(),
            Token::SamaDengan => "tanda cek kesamaan".to_string(),
            Token::TidakSamaDengan => "tanda tidak sama dengan".to_string(),
            Token::KurangDari => "tanda kurang dari".to_string(),
            Token::LebihDari => "tanda lebih dari".to_string(),
            Token::Tambah => "tanda tambah '+'".to_string(),
            Token::Kurang => "tanda kurang '-'".to_string(),
            Token::Kali => "tanda kali '*'".to_string(),
            Token::Bagi => "tanda bagi '/'".to_string(),
            Token::Pangkat => "tanda pangkat '^'".to_string(),
            Token::Panah => "tanda '=>'".to_string(),
            Token::TitikKoma => "tanda ';' (titik koma)".to_string(),
            Token::Koma => "tanda ',' (koma)".to_string(),
            Token::TitikDua => "tanda ':' (titik dua)".to_string(),
            Token::Titik => "tanda '.' (titik)".to_string(),
            Token::KurungBuka => "tanda '(' (kurung buka)".to_string(),
            Token::KurungTutup => "kurung tutup ')'".to_string(),
            Token::KurawalBuka => "kurawal buka '{' atau 'maka'".to_string(),
            Token::KurawalTutup => "kurawal tutup '}' atau 'selesai'".to_string(),
            Token::TandaTanya => "tanda tanya '?'".to_string(),
            Token::TandaTanyaGanda => "tanda tanya ganda '??'".to_string(),
            Token::BitwiseAtau => "bitwise atau '|'".to_string(),
            Token::KurungSikuBuka => "'['".to_string(),
            Token::KurungSikuTutup => "']'".to_string(),

            Token::EOF => "akhir file".to_string(),
            Token::Coba => "'coba'".to_string(),
            Token::Tangkap => "'tangkap'".to_string(),
            Token::Lempar => "'lempar'".to_string(),
            _ => format!("{:?}", self),
        }
    }

    pub fn dari_keyword(k: &str) -> Option<Token> {
        match k {
            "buat" => Some(Token::Buat),
            "tetap" => Some(Token::Tetap),
            "tampilkan" => Some(Token::Tampilkan),
            "cetak" => Some(Token::Cetak),
            "masukkan" => Some(Token::Masukkan),
            "jika" => Some(Token::Jika),
            "fungsi" => Some(Token::Fungsi),
            "kembalikan" => Some(Token::Kembalikan),
            "selama" => Some(Token::Selama),
            "ulangi" => Some(Token::Ulangi),
            "setiap" | "tiap" => Some(Token::Setiap),
            "di" | "dari" => Some(Token::Di),
            "dengan" => Some(Token::Dengan),
            "indeks" => Some(Token::Indeks),
            "berhenti" => Some(Token::Berhenti),
            "lanjut" => Some(Token::Lanjut),
            "impor" | "gabung" | "pakai" => Some(Token::Impor),
            "benar" | "true" => Some(Token::Benar),
            "salah" | "false" => Some(Token::Salah),
            "kosong" => Some(Token::Kosong),
            "maka" => Some(Token::Maka),
            "selesai" => Some(Token::Selesai),
            "minimal" => Some(Token::Minimal),
            "maksimal" => Some(Token::Maksimal),
            "dan" => Some(Token::Dan),
            "atau" => Some(Token::Atau),
            "bukan" => Some(Token::Bukan),
            "adalah" | "isinya" | "hasilnya" => Some(Token::SamaDengan),
            "coba" => Some(Token::Coba),
            "tangkap" => Some(Token::Tangkap),
            "lempar" => Some(Token::Lempar),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn lokasi(&self) -> Lokasi {
        Lokasi::from(self.span)
    }
}

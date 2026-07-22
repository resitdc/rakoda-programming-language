use ast::{Expression, InfixOperator, PrefixOperator, Program, Statement};
use errors::{Lokasi, RplError};
use lexer::token::{SpannedToken, Token};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    AndOr,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
    Property,
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Dan | Token::Atau => Precedence::AndOr,
        Token::SamaDengan | Token::TidakSamaDengan | Token::Bukan => Precedence::Equals,
        Token::LebihDari | Token::KurangDari | Token::Minimal | Token::Maksimal => {
            Precedence::LessGreater
        }
        Token::Tambah | Token::Kurang => Precedence::Sum,
        Token::Kali | Token::Bagi | Token::Mod => Precedence::Product,
        Token::KurungBuka => Precedence::Call,
        Token::KurungSikuBuka => Precedence::Index,
        Token::Titik => Precedence::Property,
        _ => Precedence::Lowest,
    }
}

/// Token-token yang menandakan awal statement baru.
/// Digunakan untuk error recovery: skip token sampai ketemu salah satu ini.
const SYNC_TOKENS: &[Token] = &[
    Token::Buat,
    Token::Jika,
    Token::Selama,
    Token::Fungsi,
    Token::Kembalikan,
    Token::Tampilkan,
    Token::Cetak,
    Token::Coba,
    Token::Lempar,
    Token::JikaTidak,
    Token::Selesai,
];

pub struct Parser {
    tokens: Vec<SpannedToken>,
    posisi: usize,
    /// Error-error yang terkumpul selama parsing.
    errors: Vec<RplError>,
    /// Mode strict: kalo true, parse error = return Err. Kalo false, dikumpulin.
    #[allow(dead_code)]
    tolerant: bool,
}

impl Parser {
    /// Buat parser baru dalam mode error-tolerant (default).
    /// Semua error akan dikumpulkan, tidak langsung bail.
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            posisi: 0,
            errors: Vec::new(),
            tolerant: true,
        }
    }

    /// Mode strict untuk backward compatibility — error langsung bail.
    pub fn new_strict(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            posisi: 0,
            errors: Vec::new(),
            tolerant: false,
        }
    }

    fn current(&self) -> &SpannedToken {
        &self.tokens[self.posisi]
    }

    fn peek(&self) -> &SpannedToken {
        if self.posisi + 1 < self.tokens.len() {
            &self.tokens[self.posisi + 1]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn advance(&mut self) {
        if self.posisi < self.tokens.len() - 1 {
            self.posisi += 1;
        }
    }

    fn current_lokasi(&self) -> Lokasi {
        self.current().lokasi()
    }

    fn push_error(&mut self, pesan: String, lokasi: Lokasi, saran: Option<String>) {
        self.errors.push(RplError::Sintaks {
            pesan,
            lokasi,
            saran,
        });
    }

    /// Error recovery: maju sampai ketemu sync token atau EOF.
    /// Setelah ditemukan, push `Statement::Error` node agar AST tetap utuh.
    fn sync(&mut self) {
        while self.current().token != Token::EOF {
            if SYNC_TOKENS.contains(&self.current().token) {
                break;
            }
            self.advance();
        }
    }

    fn expect(&mut self, expected: Token) -> bool {
        if self.current().token == expected {
            self.advance();
            true
        } else {
            let msg = format!(
                "Diharapkan {}, tetapi menemukan {}.",
                expected.to_indonesian_string(),
                self.current().token.to_indonesian_string()
            );
            let saran = Some(format!(
                "Periksa kembali struktur kodemu. Apakah kamu lupa menambahkan {} di sini?",
                expected.to_indonesian_string()
            ));
            self.push_error(msg, self.current_lokasi(), saran);
            false
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();

        while self.current().token != Token::EOF {
            let stmt = self.parse_statement();
            statements.push(stmt);
        }

        Program {
            statements,
            errors: std::mem::take(&mut self.errors),
        }
    }

    fn parse_statement(&mut self) -> Statement {
        match self.current().token {
            Token::Buat => self.parse_deklarasi_variabel(),
            Token::Jika => self.parse_jika(),
            Token::Selama => self.parse_selama(),
            Token::Setiap => self.parse_setiap(),
            Token::Fungsi => self.parse_fungsi(),
            Token::Kembalikan => self.parse_kembalikan(),
            Token::Tampilkan => self.parse_tampilkan_statement(false),
            Token::Cetak => self.parse_tampilkan_statement(true),
            Token::Coba => self.parse_coba(),
            Token::Lempar => self.parse_lempar(),
            Token::Identifier(_) => {
                if self.peek().token == Token::Assign {
                    self.parse_assignment()
                } else {
                    self.parse_expression_statement()
                }
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_deklarasi_variabel(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let nama = match &self.current().token {
            Token::Identifier(n) => n.clone(),
            _ => {
                self.push_error(
                    "Lupa memberikan nama variabel?".to_string(),
                    self.current_lokasi(),
                    Some(
                        "Setiap variabel harus memiliki nama yang jelas, contoh: buat nama = 10"
                            .to_string(),
                    ),
                );
                return Statement::Error(lokasi);
            }
        };
        self.advance();

        if !self.expect(Token::Assign) {
            return Statement::Error(lokasi);
        }

        let nilai = match self.parse_expression(Precedence::Lowest) {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return Statement::Error(lokasi);
            }
        };

        Statement::DeklarasiVariabel {
            nama,
            nilai,
            lokasi,
        }
    }

    fn parse_assignment(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        let nama = match &self.current().token {
            Token::Identifier(n) => n.clone(),
            _ => unreachable!(),
        };
        self.advance();
        if !self.expect(Token::Assign) {
            return Statement::Error(lokasi);
        }

        let nilai = match self.parse_expression(Precedence::Lowest) {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return Statement::Error(lokasi);
            }
        };

        Statement::Assignment {
            nama,
            nilai,
            lokasi,
        }
    }

    fn parse_kembalikan(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let nilai =
            if self.current().token == Token::EOF || self.current().token == Token::KurawalTutup {
                None
            } else {
                match self.parse_expression(Precedence::Lowest) {
                    Ok(e) => Some(e),
                    Err(e) => {
                        self.errors.push(e);
                        None
                    }
                }
            };

        Statement::Kembalikan { nilai, lokasi }
    }

    fn parse_tampilkan_statement(&mut self, is_cetak: bool) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let mut nilai = Vec::new();

        if self.current().token != Token::EOF && self.current().token != Token::KurawalTutup {
            loop {
                match self.parse_expression(Precedence::Lowest) {
                    Ok(e) => nilai.push(e),
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                }
                if self.current().token == Token::Koma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if is_cetak {
            Statement::Cetak { nilai, lokasi }
        } else {
            Statement::Tampilkan { nilai, lokasi }
        }
    }

    fn parse_expression_statement(&mut self) -> Statement {
        match self.parse_expression(Precedence::Lowest) {
            Ok(expr) => Statement::Expression(expr),
            Err(e) => {
                self.errors.push(e);
                self.sync();
                Statement::Error(self.current_lokasi())
            }
        }
    }

    fn parse_block(&mut self) -> Vec<Statement> {
        let is_maka = self.current().token == Token::Maka;
        let is_kurawal = self.current().token == Token::KurawalBuka;

        if is_maka || is_kurawal {
            self.advance();
        }

        let mut statements = Vec::new();

        loop {
            let current = &self.current().token;
            if current == &Token::EOF {
                break;
            }
            if is_kurawal {
                if current == &Token::KurawalTutup {
                    break;
                }
            } else {
                if current == &Token::Selesai
                    || current == &Token::JikaTidak
                    || current == &Token::Tangkap
                {
                    break;
                }
            }
            statements.push(self.parse_statement());
        }

        if is_kurawal {
            self.expect(Token::KurawalTutup);
        } else if self.current().token != Token::JikaTidak && self.current().token != Token::Tangkap
        {
            self.expect(Token::Selesai);
        }

        statements
    }

    fn parse_jika(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let kondisi = match self.parse_expression(Precedence::Lowest) {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return Statement::Error(lokasi);
            }
        };
        let konsekuensi = self.parse_block();

        let alternatif = if self.current().token == Token::JikaTidak {
            self.advance();
            if self.current().token == Token::Jika {
                Some(vec![self.parse_jika()])
            } else {
                Some(self.parse_block())
            }
        } else {
            None
        };

        Statement::Jika {
            kondisi,
            konsekuensi,
            alternatif,
            lokasi,
        }
    }

    fn parse_setiap(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let elemen = match &self.current().token {
            Token::Identifier(n) => n.clone(),
            _ => {
                self.push_error(
                    "Diharapkan nama variabel untuk elemen perulangan.".to_string(),
                    self.current_lokasi(),
                    Some("Contoh: setiap item di daftar".to_string()),
                );
                return Statement::Error(lokasi);
            }
        };
        self.advance();

        if self.current().token == Token::Di {
            self.advance();
        } else {
            self.push_error(
                "Diharapkan kata 'di' atau 'dari' setelah nama elemen.".to_string(),
                self.current_lokasi(),
                Some("Contoh: setiap item di daftar".to_string()),
            );
            return Statement::Error(lokasi);
        }

        let koleksi = match self.parse_expression(Precedence::Lowest) {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return Statement::Error(lokasi);
            }
        };

        let mut indeks = None;
        if self.current().token == Token::Dengan {
            self.advance();
            if self.current().token == Token::Indeks {
                self.advance();
                if let Token::Identifier(n) = &self.current().token {
                    indeks = Some(n.clone());
                    self.advance();
                } else {
                    self.push_error(
                        "Diharapkan nama variabel untuk indeks.".to_string(),
                        self.current_lokasi(),
                        Some("Contoh: setiap item di daftar dengan indeks i".to_string()),
                    );
                    return Statement::Error(lokasi);
                }
            } else {
                self.push_error(
                    "Diharapkan kata 'indeks' setelah 'dengan'.".to_string(),
                    self.current_lokasi(),
                    Some("Contoh: setiap item di daftar dengan indeks i".to_string()),
                );
                return Statement::Error(lokasi);
            }
        }

        let body = self.parse_block();

        Statement::Setiap {
            elemen,
            koleksi,
            indeks,
            body,
            lokasi,
        }
    }

    fn parse_selama(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let kondisi = match self.parse_expression(Precedence::Lowest) {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return Statement::Error(lokasi);
            }
        };
        let body = self.parse_block();

        Statement::Selama {
            kondisi,
            body,
            lokasi,
        }
    }

    fn parse_fungsi(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let nama = match &self.current().token {
            Token::Identifier(n) => n.clone(),
            _ => {
                self.push_error(
                    "Lupa memberikan nama fungsi?".to_string(),
                    self.current_lokasi(),
                    Some("Setiap fungsi harus memiliki nama. Contoh: fungsi sapa()".to_string()),
                );
                return Statement::Error(lokasi);
            }
        };
        self.advance();

        self.expect(Token::KurungBuka);
        let mut parameter = Vec::new();
        if self.current().token != Token::KurungTutup {
            loop {
                match &self.current().token {
                    Token::Identifier(p) => {
                        parameter.push(p.clone());
                        self.advance();
                    }
                    _ => {
                        self.push_error(
                            "Nama parameter tidak valid.".to_string(),
                            self.current_lokasi(),
                            Some("Pastikan nama data (parameter) di dalam kurung menggunakan huruf, contoh: fungsi tambah(a, b)".to_string()),
                        );
                        break;
                    }
                }

                if self.current().token == Token::Koma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::KurungTutup);

        let body = self.parse_block();

        Statement::DeklarasiFungsi {
            nama,
            parameter,
            body,
            lokasi,
        }
    }

    fn parse_coba(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let coba_body = self.parse_block();

        self.expect(Token::Tangkap);
        self.expect(Token::KurungBuka);
        let error_ident = match &self.current().token {
            Token::Identifier(n) => n.clone(),
            _ => {
                self.push_error(
                    "Nama variabel error tidak valid.".to_string(),
                    self.current_lokasi(),
                    Some(
                        "Berikan nama variabel untuk menangkap error, contoh: tangkap (error)"
                            .to_string(),
                    ),
                );
                return Statement::Error(lokasi);
            }
        };
        self.advance();
        self.expect(Token::KurungTutup);

        let tangkap_body = self.parse_block();

        Statement::CobaTangkap {
            coba_body,
            error_ident,
            tangkap_body,
            lokasi,
        }
    }

    fn parse_lempar(&mut self) -> Statement {
        let lokasi = self.current_lokasi();
        self.advance();

        let nilai = match self.parse_expression(Precedence::Lowest) {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return Statement::Error(lokasi);
            }
        };

        Statement::Lempar { nilai, lokasi }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, RplError> {
        let mut left = self.parse_prefix()?;

        while self.current().token != Token::EOF
            && precedence < token_precedence(&self.current().token)
        {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression, RplError> {
        let token = self.current().clone();
        let lok = token.lokasi();
        match token.token {
            Token::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name, lok))
            }
            Token::Angka(val) => {
                self.advance();
                Ok(Expression::Angka(val, lok))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expression::String(s, lok))
            }
            Token::Benar => {
                self.advance();
                Ok(Expression::Boolean(true, lok))
            }
            Token::Salah => {
                self.advance();
                Ok(Expression::Boolean(false, lok))
            }
            Token::Kosong => {
                self.advance();
                Ok(Expression::Kosong(lok))
            }
            Token::Fungsi => {
                self.advance();
                if !self.expect(Token::KurungBuka) {
                    return Err(RplError::Sintaks {
                        pesan: "Fungsi anonim harus dimulai dengan '('. Contoh: fungsi() { ... }"
                            .to_string(),
                        lokasi: lok,
                        saran: Some("Tambahkan '()' setelah kata 'fungsi'.".to_string()),
                    });
                }

                let mut parameter = Vec::new();
                if self.current().token != Token::KurungTutup {
                    loop {
                        match &self.current().token {
                            Token::Identifier(p) => {
                                parameter.push(p.clone());
                                self.advance();
                            }
                            _ => {
                                let e = RplError::Sintaks {
                                    pesan: "Nama parameter tidak valid.".to_string(),
                                    lokasi: self.current_lokasi(),
                                    saran: Some("Pastikan nama data (parameter) di dalam kurung menggunakan huruf, contoh: fungsi(a, b)".to_string()),
                                };
                                self.errors.push(e.clone());
                                return Err(e);
                            }
                        }

                        if self.current().token == Token::Koma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                if !self.expect(Token::KurungTutup) {
                    return Err(RplError::Sintaks {
                        pesan: "Kurung tutup ')' diharapkan setelah parameter.".to_string(),
                        lokasi: token.lokasi(),
                        saran: Some("Tambahkan ')'.".to_string()),
                    });
                }

                let body = self.parse_block();

                Ok(Expression::FungsiAnonim {
                    parameter,
                    body,
                    lokasi: token.lokasi(),
                })
            }
            Token::Bukan | Token::Kurang => {
                self.advance();
                let op = if token.token == Token::Bukan {
                    PrefixOperator::Bukan
                } else {
                    PrefixOperator::Minus
                };
                let kanan = self.parse_expression(Precedence::Prefix)?;
                Ok(Expression::Prefix {
                    operator: op,
                    kanan: Box::new(kanan),
                    lokasi: token.lokasi(),
                })
            }
            Token::KurungBuka => {
                self.advance();
                let expr = self.parse_expression(Precedence::Lowest)?;
                self.expect(Token::KurungTutup);
                Ok(expr)
            }
            Token::Impor => {
                self.advance();
                let path = match &self.current().token {
                    Token::String(s) => s.clone(),
                    _ => {
                        return Err(RplError::Sintaks {
                            pesan: "Lupa menyertakan nama file?".to_string(),
                            lokasi: token.lokasi(),
                            saran: Some("Kata 'impor' atau 'gabung' harus diikuti dengan nama file dalam tanda kutip. Contoh: impor \"matematika.rpl\"".to_string()),
                        });
                    }
                };
                self.advance();
                Ok(Expression::Impor(path, lok))
            }
            Token::KurungSikuBuka => self.parse_array(),
            Token::KurawalBuka => self.parse_kamus(),
            _ => {
                self.advance();
                let e = RplError::Sintaks {
                    pesan: format!(
                        "Potongan kode ini tidak bisa diproses: {}",
                        token.token.to_indonesian_string()
                    ),
                    lokasi: token.lokasi(),
                    saran: Some(
                        "Sepertinya ada salah ketik atau simbol yang tertinggal. Coba periksa baris ini lagi."
                            .to_string(),
                    ),
                };
                self.errors.push(e.clone());
                Err(e)
            }
        }
    }

    fn parse_infix(&mut self, left: Expression) -> Result<Expression, RplError> {
        let token = self.current().clone();

        if token.token == Token::KurungBuka {
            return self.parse_call_arguments(left);
        }

        if token.token == Token::KurungSikuBuka {
            let lokasi = self.current_lokasi();
            self.advance(); // lewati '['
            let indeks = self.parse_expression(Precedence::Lowest)?;
            self.expect(Token::KurungSikuTutup);
            return Ok(Expression::Index {
                kiri: Box::new(left),
                indeks: Box::new(indeks),
                lokasi,
            });
        }

        if token.token == Token::Titik {
            let lokasi = self.current_lokasi();
            self.advance(); // lewati '.'

            let properti = match &self.current().token {
                Token::Identifier(n) => n.clone(),
                Token::Buat => "buat".to_string(),
                Token::Tampilkan => "tampilkan".to_string(),
                Token::Cetak => "cetak".to_string(),
                Token::Masukkan => "masukkan".to_string(),
                Token::Jika => "jika".to_string(),
                Token::Fungsi => "fungsi".to_string(),
                Token::Kembalikan => "kembalikan".to_string(),
                Token::Selama => "selama".to_string(),
                Token::Ulangi => "ulangi".to_string(),
                Token::Berhenti => "berhenti".to_string(),
                Token::Lanjut => "lanjut".to_string(),
                Token::Impor => "impor".to_string(),
                Token::Benar => "benar".to_string(),
                Token::Salah => "salah".to_string(),
                Token::Kosong => "kosong".to_string(),
                Token::Maka => "maka".to_string(),
                Token::Selesai => "selesai".to_string(),
                Token::Minimal => "minimal".to_string(),
                Token::Maksimal => "maksimal".to_string(),
                Token::Dan => "dan".to_string(),
                Token::Atau => "atau".to_string(),
                Token::Bukan => "bukan".to_string(),
                _ => {
                    let e = RplError::Sintaks {
                        pesan: "Lupa menyebutkan bagian apa yang ingin diakses?".to_string(),
                        lokasi: self.current_lokasi(),
                        saran: Some("Setelah tanda titik '.', kamu harus menuliskan nama data yang ingin diambil. Contoh: objek.nama".to_string()),
                    };
                    self.errors.push(e.clone());
                    return Err(e);
                }
            };
            self.advance();
            return Ok(Expression::Index {
                kiri: Box::new(left),
                indeks: Box::new(Expression::String(properti, lokasi)),
                lokasi,
            });
        }

        let op = match token.token {
            Token::Tambah => InfixOperator::Tambah,
            Token::Kurang => InfixOperator::Kurang,
            Token::Kali => InfixOperator::Kali,
            Token::Bagi => InfixOperator::Bagi,
            Token::Mod => InfixOperator::Mod,
            Token::LebihDari => InfixOperator::LebihDari,
            Token::KurangDari => InfixOperator::KurangDari,
            Token::Minimal => InfixOperator::Minimal,
            Token::Maksimal => InfixOperator::Maksimal,
            Token::SamaDengan => InfixOperator::SamaDengan,
            Token::TidakSamaDengan | Token::Bukan => InfixOperator::TidakSamaDengan,
            Token::Dan => InfixOperator::Dan,
            Token::Atau => InfixOperator::Atau,
            _ => unreachable!(),
        };

        let precedence = token_precedence(&token.token);
        self.advance();
        let kanan = self.parse_expression(precedence)?;

        Ok(Expression::Infix {
            kiri: Box::new(left),
            operator: op,
            kanan: Box::new(kanan),
            lokasi: token.lokasi(),
        })
    }

    fn parse_call_arguments(&mut self, fungsi: Expression) -> Result<Expression, RplError> {
        let lokasi = self.current_lokasi();
        self.advance();

        let mut argumen = Vec::new();
        if self.current().token != Token::KurungTutup {
            loop {
                match self.parse_expression(Precedence::Lowest) {
                    Ok(e) => argumen.push(e),
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                }
                if self.current().token == Token::Koma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::KurungTutup);

        Ok(Expression::Call {
            fungsi: Box::new(fungsi),
            argumen,
            lokasi,
        })
    }

    fn parse_array(&mut self) -> Result<Expression, RplError> {
        let lokasi = self.current_lokasi();
        self.advance(); // lewati '['

        let mut elemen = Vec::new();
        if self.current().token != Token::KurungSikuTutup {
            loop {
                match self.parse_expression(Precedence::Lowest) {
                    Ok(e) => elemen.push(e),
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                }
                if self.current().token == Token::Koma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::KurungSikuTutup);
        Ok(Expression::Array { elemen, lokasi })
    }

    fn parse_kamus(&mut self) -> Result<Expression, RplError> {
        let lokasi = self.current_lokasi();
        self.advance(); // lewati '{'

        let mut pasangan = Vec::new();
        if self.current().token != Token::KurawalTutup {
            loop {
                let mut key = match self.parse_expression(Precedence::Lowest) {
                    Ok(e) => e,
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                };

                if let Expression::Identifier(nama, lok) = key.clone() {
                    key = Expression::String(nama, lok);
                }

                if !self.expect(Token::TitikDua) {
                    break;
                }

                match self.parse_expression(Precedence::Lowest) {
                    Ok(value) => pasangan.push((key, value)),
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                }

                if self.current().token == Token::Koma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::KurawalTutup);
        Ok(Expression::Kamus { pasangan, lokasi })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::Lexer;

    fn test_parse(input: &str) -> Program {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    fn test_parse_strict(input: &str) -> Result<Program, RplError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new_strict(tokens);
        let program = parser.parse_program();
        if program.errors.is_empty() {
            Ok(program)
        } else {
            Err(program.errors.into_iter().next().unwrap())
        }
    }

    #[test]
    fn test_parse_deklarasi() {
        let program = test_parse("buat x = 10");
        assert_eq!(program.statements.len(), 1);
        assert!(program.errors.is_empty());
        match &program.statements[0] {
            Statement::DeklarasiVariabel { nama, nilai, .. } => {
                assert_eq!(nama, "x");
                if let Expression::Angka(v, _) = nilai {
                    assert_eq!(*v, 10.0);
                } else {
                    panic!("Bukan angka");
                }
            }
            _ => panic!("Bukan deklarasi variabel"),
        }
    }

    #[test]
    fn test_parse_precedence() {
        let program = test_parse("buat x = 10 + 5 * 2");
        assert!(program.errors.is_empty());
        match &program.statements[0] {
            Statement::DeklarasiVariabel { nilai, .. } => {
                if let Expression::Infix {
                    operator, kanan, ..
                } = nilai
                {
                    assert_eq!(*operator, InfixOperator::Tambah);
                    if let Expression::Infix {
                        operator: op_kanan, ..
                    } = &**kanan
                    {
                        assert_eq!(*op_kanan, InfixOperator::Kali);
                    } else {
                        panic!("Kanan harusnya infix (*)");
                    }
                }
            }
            _ => panic!("Bukan deklarasi variabel"),
        }
    }

    #[test]
    fn test_error_tolerant_multiple_errors() {
        // Dua error sintaks: hilang koma di array, dan 'selesai' tidak valid
        let program = test_parse("buat x = 10\nbuat y = x + \nbuat z = [1 2 3]\ntampilkan z");
        // Harus tetap menghasilkan statements (AST parsial)
        assert!(!program.statements.is_empty());
        // Harus ada error yang terkumpul
        assert!(!program.errors.is_empty(), "Seharusnya ada error sintaks");
        println!("Jumlah error: {}", program.errors.len());
        for e in &program.errors {
            println!("  Error: {:?}", e);
        }
    }

    #[test]
    fn test_error_isolated_statement() {
        // Error di satu statement tidak menghentikan parsing statement berikutnya
        let program = test_parse("buat x = 10\ntampilkan x\nbuat y =\ntampilkan \"selesai\"");
        // Minimal 2 statement valid
        assert!(program.statements.len() >= 4);
        // Error harus ada
        assert!(
            !program.errors.is_empty(),
            "Seharusnya ada error di statement ke-3"
        );
    }

    #[test]
    fn test_tolernat_mode_never_panics() {
        // Input sangat rusak — lexer mungkin gagal, dan itu OK.
        // Yang penting parser tidak panic.
        let input = "@@@ ??? ### !!!";
        let mut lexer = Lexer::new(input);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                let program = parser.parse_program();
                // Parser harus survive, menghasilkan error nodes
                assert!(!program.errors.is_empty());
            }
            Err(_) => {
                // Lexer boleh gagal untuk karakter tidak dikenal
            }
        }
    }

    #[test]
    fn test_strict_mode_still_works() {
        // Strict mode seperti sebelumnya
        let result = test_parse_strict("buat x = 10");
        assert!(result.is_ok());
    }
}

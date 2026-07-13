pub mod token;
use errors::{Lokasi, RplError};
use token::{SpannedToken, Token};

pub struct Lexer {
    chars: Vec<char>,
    posisi: usize,
    baris: usize,
    kolom: usize,
    brace_count: usize,
    template_stack: Vec<usize>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            posisi: 0,
            baris: 1,
            kolom: 1,
            brace_count: 0,
            template_stack: Vec::new(),
        }
    }

    fn current_char(&self) -> Option<char> {
        self.chars.get(self.posisi).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.posisi + 1).copied()
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            if c == '\n' {
                self.baris += 1;
                self.kolom = 1;
            } else {
                self.kolom += 1;
            }
            self.posisi += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, RplError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            let c = match self.current_char() {
                Some(c) => c,
                None => break,
            };

            let lokasi_awal = Lokasi::new(self.baris, self.kolom);

            let token = match c {
                '+' => {
                    self.advance();
                    Token::Tambah
                }
                '-' => {
                    self.advance();
                    Token::Kurang
                }
                '*' => {
                    self.advance();
                    Token::Kali
                }
                '/' => {
                    if self.peek_char() == Some('/') {
                        self.advance();
                        self.advance();
                        while let Some(ch) = self.current_char() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    } else {
                        self.advance();
                        Token::Bagi
                    }
                }
                '%' => {
                    self.advance();
                    Token::Mod
                }
                '=' => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        Token::SamaDengan
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        Token::TidakSamaDengan
                    } else {
                        Token::Bukan
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        Token::Minimal
                    } else {
                        Token::LebihDari
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        Token::Maksimal
                    } else {
                        Token::KurangDari
                    }
                }
                '&' => {
                    self.advance();
                    if self.current_char() == Some('&') {
                        self.advance();
                        Token::Dan
                    } else {
                        return Err(RplError::Sintaks {
                            pesan: "Diharapkan '&&' untuk DAN.".to_string(),
                            lokasi: lokasi_awal,
                            saran: None,
                        });
                    }
                }
                '|' => {
                    self.advance();
                    if self.current_char() == Some('|') {
                        self.advance();
                        Token::Atau
                    } else {
                        return Err(RplError::Sintaks {
                            pesan: "Diharapkan '||' untuk ATAU.".to_string(),
                            lokasi: lokasi_awal,
                            saran: None,
                        });
                    }
                }
                ';' => {
                    self.advance();
                    Token::TitikKoma
                }
                ',' => {
                    self.advance();
                    Token::Koma
                }
                ':' => {
                    self.advance();
                    Token::TitikDua
                }
                '.' => {
                    self.advance();
                    Token::Titik
                }
                '(' => {
                    self.advance();
                    Token::KurungBuka
                }
                ')' => {
                    self.advance();
                    Token::KurungTutup
                }
                '[' => {
                    self.advance();
                    Token::KurungSikuBuka
                }
                ']' => {
                    self.advance();
                    Token::KurungSikuTutup
                }
                '{' => {
                    self.advance();
                    self.brace_count += 1;
                    Token::KurawalBuka
                }
                '}' => {
                    self.advance();
                    if self.brace_count > 0 {
                        self.brace_count -= 1;
                        Token::KurawalTutup
                    } else if let Some(old_count) = self.template_stack.pop() {
                        self.brace_count = old_count;
                        tokens.push(SpannedToken {
                            token: Token::KurungTutup,
                            lokasi: lokasi_awal,
                        });
                        tokens.push(SpannedToken {
                            token: Token::Tambah,
                            lokasi: lokasi_awal,
                        });
                        let tks = self.read_template_string(lokasi_awal)?;
                        tokens.extend(tks);
                        continue;
                    } else {
                        Token::KurawalTutup
                    }
                }
                '"' => self.read_string()?,
                '`' => {
                    self.advance();
                    let tks = self.read_template_string(lokasi_awal)?;
                    tokens.extend(tks);
                    continue;
                }
                _ if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(),
                _ if c.is_ascii_digit() => self.read_number()?,
                _ => {
                    let err_char = c.to_string();
                    self.advance();
                    return Err(RplError::Sintaks {
                        pesan: format!("Karakter tidak dikenali: '{}'", err_char),
                        lokasi: Lokasi::new(self.baris, self.kolom - 1),
                        saran: Some("Mungkin kamu tidak sengaja mengetik karakter ini? Pastikan hanya menggunakan simbol dan teks yang valid.".to_string()),
                    });
                }
            };

            tokens.push(SpannedToken {
                token,
                lokasi: lokasi_awal,
            });
        }

        tokens.push(SpannedToken {
            token: Token::EOF,
            lokasi: Lokasi::new(self.baris, self.kolom),
        });

        Ok(tokens)
    }

    fn read_string(&mut self) -> Result<Token, RplError> {
        let lokasi_awal = Lokasi::new(self.baris, self.kolom);
        self.advance();
        let mut string_val = String::new();

        while let Some(c) = self.current_char() {
            if c == '\\' {
                self.advance(); // lewati '\'
                if let Some(escaped) = self.current_char() {
                    match escaped {
                        'n' => string_val.push('\n'),
                        'r' => string_val.push('\r'),
                        't' => string_val.push('\t'),
                        '\\' => string_val.push('\\'),
                        '"' => string_val.push('"'),
                        _ => {
                            string_val.push('\\');
                            string_val.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else if c == '"' {
                self.advance();
                return Ok(Token::String(string_val));
            } else {
                string_val.push(c);
                self.advance();
            }
        }

        Err(RplError::Sintaks {
            pesan: "String tidak ditutup (lupa tanda kutip \")".to_string(),
            lokasi: lokasi_awal,
            saran: Some("Tambahkan tanda kutip (\") di akhir string.".to_string()),
        })
    }

    fn read_template_string(&mut self, lokasi_awal: Lokasi) -> Result<Vec<SpannedToken>, RplError> {
        let mut string_val = String::new();
        let mut tokens = Vec::new();

        while let Some(c) = self.current_char() {
            if c == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    match escaped {
                        'n' => string_val.push('\n'),
                        'r' => string_val.push('\r'),
                        't' => string_val.push('\t'),
                        '\\' => string_val.push('\\'),
                        '`' => string_val.push('`'),
                        '$' => string_val.push('$'),
                        _ => {
                            string_val.push('\\');
                            string_val.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else if c == '`' {
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::String(string_val),
                    lokasi: lokasi_awal,
                });
                return Ok(tokens);
            } else if c == '$' {
                self.advance();
                if self.current_char() == Some('{') {
                    self.advance(); // lewati '{'

                    tokens.push(SpannedToken {
                        token: Token::String(string_val),
                        lokasi: lokasi_awal,
                    });
                    tokens.push(SpannedToken {
                        token: Token::Tambah,
                        lokasi: lokasi_awal,
                    });
                    tokens.push(SpannedToken {
                        token: Token::KurungBuka,
                        lokasi: lokasi_awal,
                    });

                    self.template_stack.push(self.brace_count);
                    self.brace_count = 0;
                    return Ok(tokens);
                } else {
                    string_val.push('$');
                }
            } else {
                string_val.push(c);
                self.advance();
            }
        }

        Err(RplError::Sintaks {
            pesan: "Template literal tidak ditutup (lupa tanda backtick `)".to_string(),
            lokasi: lokasi_awal,
            saran: Some("Tambahkan tanda backtick (`) di akhir string template.".to_string()),
        })
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut text = String::new();

        while let Some(c) = self.current_char() {
            if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let simpan_posisi = self.posisi;
        let simpan_baris = self.baris;
        let simpan_kolom = self.kolom;

        if text == "jika" {
            self.skip_whitespace();
            let mut next_word = String::new();
            while let Some(c) = self.current_char() {
                if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                    next_word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if next_word == "tidak" {
                return Token::JikaTidak;
            } else {
                self.posisi = simpan_posisi;
                self.baris = simpan_baris;
                self.kolom = simpan_kolom;
            }
        } else if text == "lebih" {
            self.skip_whitespace();
            let mut next_word = String::new();
            while let Some(c) = self.current_char() {
                if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                    next_word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if next_word == "dari" {
                return Token::LebihDari;
            } else {
                self.posisi = simpan_posisi;
                self.baris = simpan_baris;
                self.kolom = simpan_kolom;
            }
        } else if text == "kurang" {
            self.skip_whitespace();
            let mut next_word = String::new();
            while let Some(c) = self.current_char() {
                if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                    next_word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if next_word == "dari" {
                return Token::KurangDari;
            } else {
                self.posisi = simpan_posisi;
                self.baris = simpan_baris;
                self.kolom = simpan_kolom;
            }
        } else if text == "sama" {
            self.skip_whitespace();
            let mut next_word = String::new();
            while let Some(c) = self.current_char() {
                if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                    next_word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if next_word == "dengan" {
                return Token::SamaDengan;
            } else {
                self.posisi = simpan_posisi;
                self.baris = simpan_baris;
                self.kolom = simpan_kolom;
            }
        } else if text == "tidak" {
            self.skip_whitespace();
            let mut next_word = String::new();
            while let Some(c) = self.current_char() {
                if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                    next_word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if next_word == "sama" {
                self.skip_whitespace();
                let mut third_word = String::new();
                while let Some(c) = self.current_char() {
                    if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                        third_word.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if third_word == "dengan" {
                    return Token::TidakSamaDengan;
                } else {
                    self.posisi = simpan_posisi;
                    self.baris = simpan_baris;
                    self.kolom = simpan_kolom;
                }
            } else {
                self.posisi = simpan_posisi;
                self.baris = simpan_baris;
                self.kolom = simpan_kolom;
            }
        }

        if let Some(token) = Token::dari_keyword(&text) {
            token
        } else {
            Token::Identifier(text)
        }
    }

    fn read_number(&mut self) -> Result<Token, RplError> {
        let lokasi_awal = Lokasi::new(self.baris, self.kolom);
        let mut number_str = String::new();
        let mut is_float = false;

        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                number_str.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                number_str.push(c);
                is_float = true;
                self.advance();
            } else {
                break;
            }
        }

        match number_str.parse::<f64>() {
            Ok(val) => Ok(Token::Angka(val)),
            Err(_) => Err(RplError::Sintaks {
                pesan: format!("Format angka tidak valid: {}", number_str),
                lokasi: lokasi_awal,
                saran: Some("Pastikan format angka benar (contoh: 123 atau 12.3).".to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let input = "buat nama = \"Restu\"";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Buat);
        assert_eq!(tokens[1].token, Token::Identifier("nama".to_string()));
        assert_eq!(tokens[2].token, Token::Assign);
        assert_eq!(tokens[3].token, Token::String("Restu".to_string()));
        assert_eq!(tokens[4].token, Token::EOF);
    }

    #[test]
    fn test_multi_word_operators() {
        let input = "jika x lebih dari 10 atau x tidak sama dengan y";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token, Token::Jika);
        assert_eq!(tokens[1].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[2].token, Token::LebihDari);
        assert_eq!(tokens[3].token, Token::Angka(10.0));
        assert_eq!(tokens[4].token, Token::Atau);
        assert_eq!(tokens[5].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[6].token, Token::TidakSamaDengan);
        assert_eq!(tokens[7].token, Token::Identifier("y".to_string()));
        assert_eq!(tokens[8].token, Token::EOF);
    }

    #[test]
    fn test_jika_tidak() {
        let input = "jika tidak { tampilkan 1 }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token, Token::JikaTidak);
        assert_eq!(tokens[1].token, Token::KurawalBuka);
        assert_eq!(tokens[2].token, Token::Tampilkan);
        assert_eq!(tokens[3].token, Token::Angka(1.0));
        assert_eq!(tokens[4].token, Token::KurawalTutup);
    }
}

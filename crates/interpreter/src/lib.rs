pub mod objek;
pub mod lingkungan;
pub mod template;
pub mod stdlib;

use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use ast::{Expression, InfixOperator, PrefixOperator, Program, Statement};
use errors::RplError;
use lingkungan::Lingkungan;
use objek::Objek;

pub struct Interpreter {
    pub lingkungan: Rc<RefCell<Lingkungan>>,
    pub output_buffer: String,
    pub capture_output: bool,
}

impl Interpreter {
    pub fn baru() -> Self {
        let lingkungan = Lingkungan::baru();
        crate::stdlib::register_all(&lingkungan);
        Self {
            lingkungan,
            output_buffer: String::new(),
            capture_output: false,
        }
    }
    
    pub fn baru_dengan_capture() -> Self {
        let lingkungan = Lingkungan::baru();
        crate::stdlib::register_all(&lingkungan);
        Self {
            lingkungan,
            output_buffer: String::new(),
            capture_output: true,
        }
    }

    pub fn baru_nested(parent: Rc<RefCell<Lingkungan>>, capture_output: bool) -> Self {
        Self {
            lingkungan: Lingkungan::baru_nested(parent),
            output_buffer: String::new(),
            capture_output,
        }
    }

    pub fn eval_program(&mut self, program: Program) -> Result<Objek, RplError> {
        let mut hasil = Objek::Kosong;

        for statement in program.statements {
            hasil = self.eval_statement(statement)?;

            if let Objek::Kembalikan(nilai) = hasil {
                return Ok(*nilai);
            }
            if let Objek::Pengecualian(err) = hasil {
                return Err(RplError::Runtime {
                    pesan: format!("Pengecualian tidak ditangkap: {}", err.to_string_pretty(0, true)),
                    lokasi: errors::Lokasi::new(0, 0),
                });
            }
        }

        Ok(hasil)
    }

    fn eval_statement(&mut self, statement: Statement) -> Result<Objek, RplError> {
        match statement {
            Statement::Expression(expr) => self.eval_expression(expr),
            Statement::Tampilkan { nilai, .. } => {
                for n in nilai {
                    let hasil_eval = self.eval_expression(n)?;
                    if self.capture_output {
                        self.output_buffer.push_str(&hasil_eval.to_string_pretty(0, true));
                        self.output_buffer.push(' ');
                    } else {
                        print!("{} ", hasil_eval.to_string_pretty(0, true));
                    }
                }
                
                if self.capture_output {
                    self.output_buffer.push('\n');
                } else {
                    println!();
                }
                Ok(Objek::Kosong)
            }
            Statement::Cetak { nilai, .. } => {
                for n in nilai {
                    let hasil_eval = self.eval_expression(n)?;
                    if self.capture_output {
                        self.output_buffer.push_str(&hasil_eval.to_string());
                    } else {
                        print!("{}", hasil_eval);
                    }
                }
                Ok(Objek::Kosong)
            }
            Statement::DeklarasiVariabel { nama, nilai, .. } => {
                let obj = self.eval_expression(nilai)?;
                self.lingkungan.borrow_mut().set(nama, obj);
                Ok(Objek::Kosong)
            }
            Statement::Assignment { nama, nilai, lokasi } => {
                if self.lingkungan.borrow().get(&nama).is_none() {
                    return Err(RplError::VariabelTidakDitemukan {
                        nama,
                        lokasi,
                        saran: Some("Pastikan menggunakan kata kunci 'buat' saat pertama kali mendeklarasikan variabel.".to_string()),
                    });
                }
                let obj = self.eval_expression(nilai)?;
                self.lingkungan.borrow_mut().set(nama, obj);
                Ok(Objek::Kosong)
            }
            Statement::Kembalikan { nilai, .. } => {
                if let Some(expr) = nilai {
                    let obj = self.eval_expression(expr)?;
                    Ok(Objek::Kembalikan(Box::new(obj)))
                } else {
                    Ok(Objek::Kembalikan(Box::new(Objek::Kosong)))
                }
            }
            Statement::Jika { kondisi, konsekuensi, alternatif, .. } => {
                let kondisi_eval = self.eval_expression(kondisi)?;
                
                if is_truthy(&kondisi_eval) {
                    self.eval_block(konsekuensi)
                } else if let Some(alt) = alternatif {
                    self.eval_block(alt)
                } else {
                    Ok(Objek::Kosong)
                }
            }
            Statement::Selama { kondisi, body, .. } => {
                let mut hasil = Objek::Kosong;
                loop {
                    let kondisi_eval = self.eval_expression(kondisi.clone())?;
                    if !is_truthy(&kondisi_eval) {
                        break;
                    }
                    hasil = self.eval_block(body.clone())?;
                    
                    if let Objek::Kembalikan(_) = hasil {
                        return Ok(hasil);
                    }
                    if let Objek::Pengecualian(_) = hasil {
                        return Ok(hasil);
                    }
                }
                Ok(hasil)
            }
            Statement::DeklarasiFungsi { nama, parameter, body, .. } => {
                let fungsi = Objek::Fungsi {
                    parameter,
                    body,
                    env: Rc::clone(&self.lingkungan),
                };
                self.lingkungan.borrow_mut().set(nama, fungsi);
                Ok(Objek::Kosong)
            }
            Statement::CobaTangkap { coba_body, error_ident, tangkap_body, .. } => {
                let mut hasil = Objek::Kosong;
                let mut terlempar = false;
                let mut nilai_error = Objek::Kosong;

                for stmt in coba_body {
                    hasil = self.eval_statement(stmt)?;
                    if let Objek::Kembalikan(_) = hasil {
                        return Ok(hasil);
                    }
                    if let Objek::Pengecualian(ref err) = hasil {
                        terlempar = true;
                        nilai_error = *err.clone();
                        break;
                    }
                }

                if terlempar {
                    // Create new environment for catch block
                    let lingkungan_lama = Rc::clone(&self.lingkungan);
                    self.lingkungan = Lingkungan::baru_nested(Rc::clone(&lingkungan_lama));
                    
                    self.lingkungan.borrow_mut().set(error_ident, nilai_error);
                    
                    let mut hasil_tangkap = Objek::Kosong;
                    for stmt in tangkap_body {
                        hasil_tangkap = self.eval_statement(stmt)?;
                        if let Objek::Kembalikan(_) = hasil_tangkap {
                            self.lingkungan = lingkungan_lama;
                            return Ok(hasil_tangkap);
                        }
                        if let Objek::Pengecualian(_) = hasil_tangkap {
                            self.lingkungan = lingkungan_lama;
                            return Ok(hasil_tangkap);
                        }
                    }
                    
                    self.lingkungan = lingkungan_lama;
                    Ok(hasil_tangkap)
                } else {
                    Ok(hasil)
                }
            }
            Statement::Lempar { nilai, .. } => {
                let eval_nilai = self.eval_expression(nilai)?;
                Ok(Objek::Pengecualian(Box::new(eval_nilai)))
            }
        }
    }

    fn eval_block(&mut self, statements: Vec<Statement>) -> Result<Objek, RplError> {
        let mut hasil = Objek::Kosong;

        for statement in statements {
            hasil = self.eval_statement(statement)?;

            if let Objek::Kembalikan(_) = hasil {
                return Ok(hasil);
            }
            if let Objek::Pengecualian(_) = hasil {
                return Ok(hasil);
            }
        }

        Ok(hasil)
    }

    fn eval_expression(&mut self, expr: Expression) -> Result<Objek, RplError> {
        match expr {
            Expression::Angka(val, _) => Ok(Objek::Angka(val)),
            Expression::String(val, _) => Ok(Objek::String(val)),
            Expression::Boolean(val, _) => Ok(Objek::Boolean(val)),
            Expression::Kosong(_) => Ok(Objek::Kosong),
            Expression::FungsiAnonim { parameter, body, .. } => {
                Ok(Objek::Fungsi {
                    parameter,
                    body,
                    env: Rc::clone(&self.lingkungan),
                })
            }
            Expression::Identifier(nama, lokasi) => {
                match self.lingkungan.borrow().get(&nama) {
                    Some(val) => Ok(val),
                    None => Err(RplError::VariabelTidakDitemukan { 
                        nama: nama.clone(),
                        lokasi,
                        saran: Some(format!("Gunakan perintah 'buat {} = ...' terlebih dahulu sebelum memakainya.", nama)),
                    }),
                }
            }
            Expression::Impor(path_str, lokasi) => {
                let path = Path::new(&path_str);
                let kode_asli = match fs::read_to_string(path) {
                    Ok(k) => k,
                    Err(e) => return Err(RplError::Sintaks {
                        pesan: format!("Gagal memuat modul '{}': {}", path_str, e),
                        lokasi,
                        saran: Some("Pastikan path file sudah benar dan file dapat diakses.".to_string()),
                    }),
                };
                
                let is_html_template = path_str.ends_with(".rpl.html");
                let kode_sumber = if is_html_template {
                    template::preprocess_template(&kode_asli)
                } else {
                    kode_asli
                };
                
                let mut lexer = lexer::Lexer::new(&kode_sumber);
                let tokens = lexer.tokenize().map_err(|mut e| {
                    if let RplError::Sintaks { pesan, .. } = &mut e {
                        *pesan = format!("Di dalam modul '{}': {}", path_str, pesan);
                    }
                    e
                })?;
                
                let mut parser = parser::Parser::new(tokens);
                let program = parser.parse_program().map_err(|mut e| {
                    if let RplError::Sintaks { pesan, .. } = &mut e {
                        *pesan = format!("Di dalam modul '{}': {}", path_str, pesan);
                    }
                    e
                })?;
                
                let mut mod_interpreter = if is_html_template {
                    Interpreter::baru_nested(self.lingkungan.clone(), self.capture_output)
                } else {
                    Interpreter::baru()
                };
                
                mod_interpreter.eval_program(program)?;
                
                if is_html_template && self.capture_output {
                    self.output_buffer.push_str(&mod_interpreter.output_buffer);
                }
                
                let modul_obj = Objek::Modul(mod_interpreter.lingkungan);
                
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let mut real_stem = stem;
                    if real_stem.ends_with(".rpl") {
                        real_stem = real_stem.trim_end_matches(".rpl");
                    }
                    self.lingkungan.borrow_mut().set(real_stem.to_string(), modul_obj.clone());
                }
                
                Ok(modul_obj)
            }
            Expression::Array { elemen, .. } => {
                let mut hasil = Vec::new();
                for e in elemen {
                    hasil.push(self.eval_expression(e)?);
                }
                Ok(Objek::Array(Rc::new(RefCell::new(hasil))))
            }
            Expression::Kamus { pasangan, .. } => {
                let mut hasil = std::collections::HashMap::new();
                for (k_expr, v_expr) in pasangan {
                    let k = self.eval_expression(k_expr)?;
                    let v = self.eval_expression(v_expr)?;
                    
                    let k_str = match k {
                        Objek::String(s) => s,
                        _ => format!("{}", k),
                    };
                    hasil.insert(k_str, v);
                }
                Ok(Objek::Kamus(Rc::new(RefCell::new(hasil))))
            }
            Expression::Index { kiri, indeks, lokasi } => {
                let kiri_obj = self.eval_expression(*kiri)?;
                let indeks_obj = self.eval_expression(*indeks)?;
                
                match kiri_obj {
                    Objek::Array(elemen_rc) => {
                        let elemen = elemen_rc.borrow();
                        match indeks_obj {
                            Objek::Angka(val) => {
                                let idx = val as usize;
                                if val < 0.0 || idx >= elemen.len() {
                                    Err(RplError::Sintaks {
                                        pesan: format!("Indeks array {} di luar batas (0-{}).", idx, elemen.len().saturating_sub(1)),
                                        lokasi,
                                        saran: Some("Pastikan nomor yang diakses tidak melebihi ukuran array.".to_string()),
                                    })
                                } else {
                                    Ok(elemen[idx].clone())
                                }
                            }
                            _ => Err(RplError::TipeData {
                                pesan: "Akses elemen array harus menggunakan angka.".to_string(),
                                lokasi,
                                saran: Some("Gunakan angka untuk mengakses urutan di dalam array, contoh: data[0]".to_string()),
                            })
                        }
                    }
                    Objek::Kamus(pasangan_rc) => {
                        let pasangan = pasangan_rc.borrow();
                        let kunci = match indeks_obj {
                            Objek::String(s) => s,
                            _ => format!("{}", indeks_obj),
                        };
                        
                        match pasangan.get(&kunci) {
                            Some(val) => Ok(val.clone()),
                            None => Ok(Objek::Kosong),
                        }
                    }
                    Objek::Modul(env) => {
                        let kunci = match indeks_obj {
                            Objek::String(s) => s,
                            _ => format!("{}", indeks_obj),
                        };
                        match env.borrow().get(&kunci) {
                            Some(val) => Ok(val),
                            None => Err(RplError::Sintaks {
                                pesan: format!("Properti atau fungsi '{}' tidak ditemukan di dalam modul.", kunci),
                                lokasi,
                                saran: None,
                            }),
                        }
                    }
                    _ => Err(RplError::TipeData {
                        pesan: "Hanya Array, Kamus, atau Modul yang bisa diakses bagian dalamnya dengan `[ ]` atau `.`".to_string(),
                        lokasi,
                        saran: Some("Pastikan variabel tersebut adalah daftar kumpulan data.".to_string()),
                    }),
                }
            }
            Expression::Prefix { operator, kanan, lokasi } => {
                let kanan_obj = self.eval_expression(*kanan)?;
                self.eval_prefix_expression(operator, kanan_obj, lokasi)
            }
            Expression::Infix { kiri, operator, kanan, lokasi } => {
                let kiri_obj = self.eval_expression(*kiri)?;
                let kanan_obj = self.eval_expression(*kanan)?;
                self.eval_infix_expression(operator, kiri_obj, kanan_obj, lokasi)
            }
            Expression::Call { fungsi, argumen, lokasi } => {
                let nama_fungsi = match &*fungsi {
                    Expression::Identifier(nama, _) => nama.clone(),
                    _ => "fungsi_anonim".to_string(),
                };
                
                let fungsi_obj = self.eval_expression(*fungsi)?;
                
                let mut arg_eval = Vec::new();
                for arg in argumen {
                    arg_eval.push(self.eval_expression(arg)?);
                }

                match fungsi_obj {
                    Objek::FungsiBawaan(func) => {
                        Ok(func(arg_eval))
                    }
                    Objek::MetodeBawaan(func) => {
                        Ok(func(arg_eval))
                    }
                    Objek::Fungsi { parameter, body, env } => {
                        if arg_eval.len() != parameter.len() {
                            return Err(RplError::Sintaks {
                                pesan: format!("Fungsi '{}' mengharapkan {} data (argumen), tapi kamu memberikan {}.", nama_fungsi, parameter.len(), arg_eval.len()),
                                lokasi,
                                saran: Some("Pastikan jumlah data di dalam kurung sesuai dengan yang diminta fungsi.".to_string()),
                            });
                        }

                        let func_env = Lingkungan::baru_nested(env);
                        for (i, param_nama) in parameter.iter().enumerate() {
                            func_env.borrow_mut().set(param_nama.clone(), arg_eval[i].clone());
                        }

                        let env_sebelumnya = Rc::clone(&self.lingkungan);
                        self.lingkungan = func_env;

                        let hasil = self.eval_block(body);

                        self.lingkungan = env_sebelumnya;

                        match hasil? {
                            Objek::Kembalikan(val) => Ok(*val),
                            Objek::Pengecualian(val) => Ok(Objek::Pengecualian(val)),
                            _ => Ok(Objek::Kosong),
                        }
                    }
                    _ => {
                        Err(RplError::TipeData {
                            pesan: format!("'{}' bukanlah sebuah fungsi yang bisa dipanggil.", nama_fungsi),
                            lokasi,
                            saran: Some(format!("Mungkin '{}' adalah variabel biasa? Pastikan hanya menggunakan '()' pada nama fungsi.", nama_fungsi)),
                        })
                    }
                }
            }
        }
    }

    fn eval_prefix_expression(&self, operator: PrefixOperator, kanan: Objek, lokasi: errors::Lokasi) -> Result<Objek, RplError> {
        match operator {
            PrefixOperator::Bukan => {
                Ok(Objek::Boolean(!is_truthy(&kanan)))
            }
            PrefixOperator::Minus => {
                match kanan {
                    Objek::Angka(val) => Ok(Objek::Angka(-val)),
                    _ => Err(RplError::TipeData {
                        pesan: "Kamu menggunakan tanda kurang '-' (negatif) pada tipe data yang bukan Angka.".to_string(),
                        lokasi,
                        saran: Some("Pastikan kamu hanya menempelkan tanda '-' pada variabel bernilai Angka.".to_string()),
                    })
                }
            }
        }
    }

    fn eval_infix_expression(&self, operator: InfixOperator, kiri: Objek, kanan: Objek, lokasi: errors::Lokasi) -> Result<Objek, RplError> {
        match (kiri, kanan) {
            (Objek::Angka(kiri_val), Objek::Angka(kanan_val)) => {
                self.eval_angka_infix(operator, kiri_val, kanan_val, lokasi)
            }
            (Objek::String(kiri_val), Objek::String(kanan_val)) => {
                self.eval_string_infix(operator, kiri_val, kanan_val, lokasi)
            }
            (Objek::String(kiri_val), kanan_val) => {
                if operator == InfixOperator::Tambah {
                    Ok(Objek::String(format!("{}{}", kiri_val, kanan_val)))
                } else if operator == InfixOperator::SamaDengan {
                    Ok(Objek::Boolean(false)) // String and other type are never equal
                } else if operator == InfixOperator::TidakSamaDengan {
                    Ok(Objek::Boolean(true))
                } else {
                    let saran = if let Objek::Angka(_) = kanan_val {
                        Some("Gunakan fungsi `angka()` untuk mengubah Teks menjadi Angka, atau `teks()` untuk mengubah Angka menjadi Teks.".to_string())
                    } else {
                        Some("Teks hanya bisa digabungkan dengan tanda tambah '+'. Tidak bisa dikurang, dikali, atau dibandingkan (<, >).".to_string())
                    };
                    Err(RplError::TipeData {
                        pesan: format!("Operator '{}' membutuhkan tipe data yang seragam. Ditemukan: Teks dan Tipe Data Lain.", operator),
                        lokasi,
                        saran,
                    })
                }
            }
            (kiri_val, Objek::String(kanan_val)) => {
                if operator == InfixOperator::Tambah {
                    Ok(Objek::String(format!("{}{}", kiri_val, kanan_val)))
                } else if operator == InfixOperator::SamaDengan {
                    Ok(Objek::Boolean(false))
                } else if operator == InfixOperator::TidakSamaDengan {
                    Ok(Objek::Boolean(true))
                } else {
                    let saran = if let Objek::Angka(_) = kiri_val {
                        Some("Gunakan fungsi `angka()` untuk mengubah Teks menjadi Angka, atau `teks()` untuk mengubah Angka menjadi Teks.".to_string())
                    } else {
                        Some("Teks hanya bisa digabungkan dengan tanda tambah '+'. Tidak bisa dikurang, dikali, atau dibandingkan (<, >).".to_string())
                    };
                    Err(RplError::TipeData {
                        pesan: format!("Operator '{}' membutuhkan tipe data yang seragam. Ditemukan: Tipe Data Lain dan Teks.", operator),
                        lokasi,
                        saran,
                    })
                }
            }
            (kiri_obj, kanan_obj) => {
                match operator {
                    InfixOperator::SamaDengan => Ok(Objek::Boolean(kiri_obj == kanan_obj)),
                    InfixOperator::TidakSamaDengan => Ok(Objek::Boolean(kiri_obj != kanan_obj)),
                    InfixOperator::Dan => Ok(Objek::Boolean(is_truthy(&kiri_obj) && is_truthy(&kanan_obj))),
                    InfixOperator::Atau => Ok(Objek::Boolean(is_truthy(&kiri_obj) || is_truthy(&kanan_obj))),
                    _ => {
                        let (saran, pesan_spesifik) = if let (Objek::String(_), Objek::Angka(_)) | (Objek::Angka(_), Objek::String(_)) = (&kiri_obj, &kanan_obj) {
                            (
                                Some("Gunakan fungsi `angka()` untuk mengubah Teks menjadi Angka, atau `teks()` untuk mengubah Angka menjadi Teks.".to_string()),
                                format!("Operator '{}' membutuhkan tipe data yang seragam. Ditemukan: Teks dan Angka.", operator)
                            )
                        } else {
                            (
                                Some("Pastikan tipe datanya sama atau mendukung operasi tersebut (misalnya Angka dengan Angka).".to_string()),
                                format!("Kamu mencoba menghitung {} dengan {} menggunakan operator '{}'", kiri_obj, kanan_obj, operator)
                            )
                        };

                        Err(RplError::TipeData {
                            pesan: pesan_spesifik,
                            lokasi,
                            saran,
                        })
                    }
                }
            }
        }
    }

    fn eval_angka_infix(&self, operator: InfixOperator, kiri: f64, kanan: f64, lokasi: errors::Lokasi) -> Result<Objek, RplError> {
        match operator {
            InfixOperator::Tambah => Ok(Objek::Angka(kiri + kanan)),
            InfixOperator::Kurang => Ok(Objek::Angka(kiri - kanan)),
            InfixOperator::Kali => Ok(Objek::Angka(kiri * kanan)),
            InfixOperator::Bagi => {
                if kanan == 0.0 {
                    Err(RplError::TipeData {
                        pesan: "Pembagian dengan nilai 0 (Nol).".to_string(),
                        lokasi,
                        saran: Some("Pastikan angka pembaginya tidak sama dengan nol (0). Tidak ada angka yang bisa dibagi nol.".to_string()),
                    })
                } else {
                    Ok(Objek::Angka(kiri / kanan))
                }
            }
            InfixOperator::Mod => Ok(Objek::Angka(kiri % kanan)),
            InfixOperator::LebihDari => Ok(Objek::Boolean(kiri > kanan)),
            InfixOperator::KurangDari => Ok(Objek::Boolean(kiri < kanan)),
            InfixOperator::Minimal => Ok(Objek::Boolean(kiri >= kanan)),
            InfixOperator::Maksimal => Ok(Objek::Boolean(kiri <= kanan)),
            InfixOperator::SamaDengan => Ok(Objek::Boolean(kiri == kanan)),
            InfixOperator::TidakSamaDengan => Ok(Objek::Boolean(kiri != kanan)),
            _ => Ok(Objek::Kosong),
        }
    }

    fn eval_string_infix(&self, operator: InfixOperator, kiri: String, kanan: String, lokasi: errors::Lokasi) -> Result<Objek, RplError> {
        match operator {
            InfixOperator::Tambah => Ok(Objek::String(format!("{}{}", kiri, kanan))),
            InfixOperator::SamaDengan => Ok(Objek::Boolean(kiri == kanan)),
            InfixOperator::TidakSamaDengan => Ok(Objek::Boolean(kiri != kanan)),
            _ => Err(RplError::TipeData {
                pesan: "Kamu mencoba menggunakan operasi matematika yang tidak diizinkan pada Teks.".to_string(),
                lokasi,
                saran: Some("Untuk menggabungkan teks, gunakan tanda tambah '+'. Operasi lain seperti kurang, kali, atau bagi tidak berlaku untuk teks.".to_string()),
            })
        }
    }
}

fn is_truthy(obj: &Objek) -> bool {
    match obj {
        Objek::Kosong => false,
        Objek::Boolean(val) => *val,
        Objek::Angka(val) => *val != 0.0,
        _ => true,
    }
}

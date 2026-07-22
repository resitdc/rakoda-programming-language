use crate::heap::{Heap, HeapData};
use crate::opcodes::OpCode;
use crate::value::{FungsiVM, Value};
use ast::{Expression, InfixOperator, PrefixOperator, Program, Statement};
use errors::Lokasi;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub locations: Vec<Lokasi>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            locations: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, lokasi: Lokasi) {
        self.code.push(byte);
        self.locations.push(lokasi);
    }

    pub fn write_opcode(&mut self, op: OpCode, lokasi: Lokasi) {
        self.code.push(op as u8);
        self.locations.push(lokasi);
    }

    pub fn write_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    pub fn write_u16(&mut self, val: u16, lokasi: Lokasi) {
        let bytes = val.to_be_bytes();
        self.write(bytes[0], lokasi);
        self.write(bytes[1], lokasi);
    }
}

pub struct Compiler<'a> {
    pub chunk: Chunk,
    pub heap: &'a mut Heap,
    pub base_path: Option<std::path::PathBuf>,
    pub current_file: Option<String>,
}

impl<'a> Compiler<'a> {
    pub fn new(heap: &'a mut Heap) -> Self {
        Self {
            chunk: Chunk::new(),
            heap,
            base_path: None,
            current_file: None,
        }
    }

    pub fn baru_dengan_base_path(
        heap: &'a mut Heap,
        base_path: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            chunk: Chunk::new(),
            heap,
            base_path,
            current_file: None,
        }
    }

    pub fn baru_dengan_file(
        heap: &'a mut Heap,
        base_path: Option<std::path::PathBuf>,
        current_file: Option<String>,
    ) -> Self {
        Self {
            chunk: Chunk::new(),
            heap,
            base_path,
            current_file,
        }
    }

    pub fn compile(mut self, program: Program) -> Result<Chunk, String> {
        let dummy_lokasi = Lokasi { baris: 1, kolom: 1 };
        let len = program.statements.len();
        for (i, stmt) in program.statements.into_iter().enumerate() {
            if i == len - 1
                && let Statement::Expression(expr) = &stmt
            {
                self.compile_expression(expr.clone())?;
                self.chunk.write_opcode(OpCode::Return, dummy_lokasi);
                continue;
            }
            self.compile_statement(stmt)?;
        }
        if self.chunk.code.last() != Some(&(OpCode::Return as u8)) {
            self.chunk.write_opcode(OpCode::Return, dummy_lokasi);
        }
        Ok(self.chunk)
    }

    fn compile_statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => {
                let lok = *expr.lokasi();
                self.compile_expression(expr)?;
                self.chunk.write_opcode(OpCode::Pop, lok);
            }
            Statement::DeklarasiVariabel {
                nama,
                nilai,
                lokasi,
            }
            | Statement::Assignment {
                nama,
                nilai,
                lokasi,
            } => {
                self.compile_expression(nilai)?;
                let name_idx = self.heap.alloc(HeapData::String(nama));
                let const_idx = self.chunk.write_constant(Value::String(name_idx));
                self.chunk.write_opcode(OpCode::StoreVar, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Statement::Tampilkan { nilai, lokasi } => {
                for expr in nilai {
                    self.compile_expression(expr)?;
                    self.chunk.write_opcode(OpCode::Print, lokasi);
                }
            }
            Statement::Cetak { nilai, lokasi } => {
                for expr in nilai {
                    self.compile_expression(expr)?;
                    self.chunk.write_opcode(OpCode::Print, lokasi);
                }
            }
            Statement::Jika {
                kondisi,
                konsekuensi,
                alternatif,
                lokasi,
            } => {
                self.compile_expression(kondisi)?;

                let jump_if_false_offset = self.emit_jump(OpCode::JumpIfFalse, lokasi);

                for stmt in konsekuensi {
                    self.compile_statement(stmt)?;
                }

                let jump_offset = self.emit_jump(OpCode::Jump, lokasi);

                self.patch_jump(jump_if_false_offset);

                if let Some(alt) = alternatif {
                    for stmt in alt {
                        self.compile_statement(stmt)?;
                    }
                }

                self.patch_jump(jump_offset);
            }
            Statement::Selama {
                kondisi,
                body,
                lokasi,
            } => {
                let loop_start = self.chunk.code.len();

                self.compile_expression(kondisi)?;

                let jump_if_false_offset = self.emit_jump(OpCode::JumpIfFalse, lokasi);

                for stmt in body {
                    self.compile_statement(stmt)?;
                }

                let jump_back = self.emit_jump(OpCode::Jump, lokasi);
                self.patch_jump_to(jump_back, loop_start);

                self.patch_jump(jump_if_false_offset);
            }
            Statement::Setiap {
                elemen,
                koleksi,
                indeks,
                body,
                lokasi,
            } => {
                self.compile_expression(koleksi)?;
                self.chunk.write_opcode(OpCode::IterInit, lokasi);
                
                let loop_start = self.chunk.code.len();
                let jump_if_exhausted = self.emit_jump(OpCode::IterNext, lokasi);
                
                let name_idx = self.heap.alloc(crate::heap::HeapData::String(elemen));
                let const_name_idx = self.chunk.write_constant(Value::String(name_idx));
                self.chunk.write_opcode(OpCode::StoreVar, lokasi);
                self.chunk.write_u16(const_name_idx, lokasi);
                
                if let Some(idx_name) = indeks {
                    let idx_name_id = self.heap.alloc(crate::heap::HeapData::String(idx_name));
                    let const_idx_name = self.chunk.write_constant(Value::String(idx_name_id));
                    self.chunk.write_opcode(OpCode::StoreVar, lokasi);
                    self.chunk.write_u16(const_idx_name, lokasi);
                } else {
                    self.chunk.write_opcode(OpCode::Pop, lokasi);
                }

                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                let jump_back = self.emit_jump(OpCode::Jump, lokasi);
                self.patch_jump_to(jump_back, loop_start);
                
                self.patch_jump(jump_if_exhausted);
                
                self.chunk.write_opcode(OpCode::Pop, lokasi);
                self.chunk.write_opcode(OpCode::Pop, lokasi);
            }
            Statement::DeklarasiFungsi {
                nama,
                parameter,
                body,
                lokasi,
            } => {
                let mut fn_compiler = Compiler::baru_dengan_file(
                    self.heap,
                    self.base_path.clone(),
                    self.current_file.clone(),
                );
                let body_len = body.len();
                for (i, stmt) in body.into_iter().enumerate() {
                    if i == body_len - 1
                        && let Statement::Expression(expr) = &stmt
                    {
                        fn_compiler.compile_expression(expr.clone())?;
                        fn_compiler.chunk.write_opcode(OpCode::Return, lokasi);
                        continue;
                    }
                    fn_compiler.compile_statement(stmt)?;
                }

                let mut chunk = fn_compiler.chunk;
                if chunk.code.last() != Some(&(OpCode::Return as u8)) {
                    let const_idx = chunk.write_constant(Value::Kosong);
                    chunk.write_opcode(OpCode::LoadConst, lokasi);
                    chunk.write_u16(const_idx, lokasi);
                    chunk.write_opcode(OpCode::Return, lokasi);
                }

                let fungsi = FungsiVM {
                    nama: nama.clone(),
                    parameter,
                    chunk,
                    file: self.current_file.clone(),
                };
                let fungsi_idx = self.heap.alloc(HeapData::Fungsi(fungsi));
                let const_idx = self.chunk.write_constant(Value::Fungsi(fungsi_idx, 0));
                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);

                let name_idx = self.heap.alloc(HeapData::String(nama));
                let const_name_idx = self.chunk.write_constant(Value::String(name_idx));
                self.chunk.write_opcode(OpCode::StoreVar, lokasi);
                self.chunk.write_u16(const_name_idx, lokasi);
            }
            Statement::Kembalikan { nilai, lokasi } => {
                if let Some(expr) = nilai {
                    self.compile_expression(expr)?;
                } else {
                    let const_idx = self.chunk.write_constant(Value::Kosong);
                    self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                    self.chunk.write_u16(const_idx, lokasi);
                }
                self.chunk.write_opcode(OpCode::Return, lokasi);
            }
            Statement::CobaTangkap {
                coba_body,
                error_ident,
                tangkap_body,
                lokasi,
            } => {
                let setup_catch_offset = self.emit_jump(OpCode::SetupCatch, lokasi);

                for stmt in coba_body {
                    self.compile_statement(stmt)?;
                }

                self.chunk.write_opcode(OpCode::PopCatch, lokasi);
                let jump_over_catch = self.emit_jump(OpCode::Jump, lokasi);

                self.patch_jump(setup_catch_offset);

                // Here, the error value is at the top of the stack.
                // We need to store it in the error_ident variable.
                let name_idx = self.heap.alloc(HeapData::String(error_ident));
                let const_name_idx = self.chunk.write_constant(Value::String(name_idx));
                self.chunk.write_opcode(OpCode::StoreVar, lokasi);
                self.chunk.write_u16(const_name_idx, lokasi);

                for stmt in tangkap_body {
                    self.compile_statement(stmt)?;
                }

                self.patch_jump(jump_over_catch);
            }
            Statement::Lempar { nilai, lokasi } => {
                self.compile_expression(nilai)?;
                self.chunk.write_opcode(OpCode::Throw, lokasi);
            }
            Statement::Error(_) => {
                // Error recovery node — tidak menghasilkan bytecode
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: Expression) -> Result<(), String> {
        match expr {
            Expression::Angka(val, lokasi) => {
                let const_idx = self.chunk.write_constant(Value::Angka(val));
                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Expression::String(val, lokasi) => {
                let s_idx = self.heap.alloc(HeapData::String(val));
                let const_idx = self.chunk.write_constant(Value::String(s_idx));
                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Expression::Boolean(val, lokasi) => {
                let const_idx = self.chunk.write_constant(Value::Boolean(val));
                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Expression::Kosong(lokasi) => {
                let const_idx = self.chunk.write_constant(Value::Kosong);
                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Expression::Identifier(nama, lokasi) => {
                let name_idx = self.heap.alloc(HeapData::String(nama));
                let const_idx = self.chunk.write_constant(Value::String(name_idx));
                self.chunk.write_opcode(OpCode::LoadVar, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Expression::Infix {
                kiri,
                operator,
                kanan,
                lokasi,
            } => {
                self.compile_expression(*kiri)?;
                self.compile_expression(*kanan)?;
                match operator {
                    InfixOperator::Tambah => self.chunk.write_opcode(OpCode::Add, lokasi),
                    InfixOperator::Kurang => self.chunk.write_opcode(OpCode::Subtract, lokasi),
                    InfixOperator::Kali => self.chunk.write_opcode(OpCode::Multiply, lokasi),
                    InfixOperator::Bagi => self.chunk.write_opcode(OpCode::Divide, lokasi),
                    InfixOperator::Mod => self.chunk.write_opcode(OpCode::Modulus, lokasi),
                    InfixOperator::LebihDari => self.chunk.write_opcode(OpCode::Greater, lokasi),
                    InfixOperator::KurangDari => self.chunk.write_opcode(OpCode::Less, lokasi),
                    InfixOperator::Minimal => self.chunk.write_opcode(OpCode::GreaterEqual, lokasi),
                    InfixOperator::Maksimal => self.chunk.write_opcode(OpCode::LessEqual, lokasi),
                    InfixOperator::SamaDengan => self.chunk.write_opcode(OpCode::Equal, lokasi),
                    InfixOperator::TidakSamaDengan => {
                        self.chunk.write_opcode(OpCode::NotEqual, lokasi)
                    }
                    InfixOperator::Dan => self.chunk.write_opcode(OpCode::And, lokasi),
                    InfixOperator::Atau => self.chunk.write_opcode(OpCode::Or, lokasi),
                }
            }
            Expression::Prefix {
                operator,
                kanan,
                lokasi,
            } => {
                self.compile_expression(*kanan)?;
                match operator {
                    PrefixOperator::Minus => self.chunk.write_opcode(OpCode::Negate, lokasi),
                    PrefixOperator::Bukan => self.chunk.write_opcode(OpCode::Not, lokasi),
                }
            }
            Expression::Call {
                fungsi,
                argumen,
                lokasi,
            } => {
                self.compile_expression(*fungsi)?;
                let arg_count = argumen.len();
                if arg_count > 255 {
                    return Err("Argumen maksimal 255".to_string());
                }
                for arg in argumen {
                    self.compile_expression(arg)?;
                }
                self.chunk.write_opcode(OpCode::Call, lokasi);
                self.chunk.write(arg_count as u8, lokasi);
            }
            Expression::Index {
                kiri,
                indeks,
                lokasi,
            } => {
                self.compile_expression(*kiri)?;
                self.compile_expression(*indeks)?;
                self.chunk.write_opcode(OpCode::GetIndex, lokasi);
            }
            Expression::Array { elemen, lokasi } => {
                let count = elemen.len();
                if count > u16::MAX as usize {
                    return Err("Elemen array maksimal 65535".to_string());
                }
                for el in elemen {
                    self.compile_expression(el)?;
                }
                self.chunk.write_opcode(OpCode::MakeArray, lokasi);
                self.chunk.write_u16(count as u16, lokasi);
            }
            Expression::Kamus { pasangan, lokasi } => {
                let count = pasangan.len();
                if count > u16::MAX as usize {
                    return Err("Elemen kamus maksimal 65535".to_string());
                }
                for (k, v) in pasangan {
                    self.compile_expression(k)?;
                    self.compile_expression(v)?;
                }
                self.chunk.write_opcode(OpCode::MakeKamus, lokasi);
                self.chunk.write_u16(count as u16, lokasi);
            }
            Expression::FungsiAnonim {
                parameter,
                body,
                lokasi,
            } => {
                let mut fn_compiler = Compiler::baru_dengan_file(
                    self.heap,
                    self.base_path.clone(),
                    self.current_file.clone(),
                );
                let body_len = body.len();
                for (i, stmt) in body.into_iter().enumerate() {
                    if i == body_len - 1
                        && let Statement::Expression(expr) = &stmt
                    {
                        fn_compiler.compile_expression(expr.clone())?;
                        fn_compiler.chunk.write_opcode(OpCode::Return, lokasi);
                        continue;
                    }
                    fn_compiler.compile_statement(stmt)?;
                }

                let mut chunk = fn_compiler.chunk;
                if chunk.code.last() != Some(&(OpCode::Return as u8)) {
                    let const_idx = chunk.write_constant(Value::Kosong);
                    chunk.write_opcode(OpCode::LoadConst, lokasi);
                    chunk.write_u16(const_idx, lokasi);
                    chunk.write_opcode(OpCode::Return, lokasi);
                }

                let fungsi = crate::value::FungsiVM {
                    nama: "<anonim>".to_string(),
                    parameter,
                    chunk,
                    file: self.current_file.clone(),
                };
                let fungsi_idx = self.heap.alloc(crate::heap::HeapData::Fungsi(fungsi));
                let const_idx = self.chunk.write_constant(Value::Fungsi(fungsi_idx, 0));
                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
            }
            Expression::Impor(path_str, lokasi) => {
                let resolved_path = if let Some(base) = &self.base_path {
                    if std::path::Path::new(&path_str).is_relative() {
                        base.join(&path_str)
                    } else {
                        std::path::PathBuf::from(&path_str)
                    }
                } else {
                    std::path::PathBuf::from(&path_str)
                };

                let (kode_asli, final_path) = match std::fs::read_to_string(&resolved_path) {
                    Ok(k) => (k, resolved_path),
                    Err(e) => {
                        // Fallback ke rpl_modules
                        let mut current_dir = std::env::current_dir().unwrap_or_default();
                        if let Some(base) = &self.base_path {
                            current_dir = base.clone();
                        }

                        let module_path = current_dir
                            .join("rpl_modules")
                            .join(&path_str)
                            .join("main.rpl");
                        match std::fs::read_to_string(&module_path) {
                            Ok(k) => (k, module_path),
                            Err(_) => {
                                return Err(format!(
                                    "Gagal memuat modul '{}': {}",
                                    resolved_path.display(),
                                    e
                                ));
                            }
                        }
                    }
                };

                let is_html_template = path_str.ends_with(".rpl.html");
                let kode_sumber = if is_html_template {
                    stdlib::template::preprocess_template(&kode_asli)
                } else {
                    kode_asli
                };

                let mut lexer = lexer::Lexer::new(&kode_sumber);
                let tokens = lexer
                    .tokenize()
                    .map_err(|e| format!("Error lexer di '{}': {:?}", path_str, e))?;

                let mut parser = parser::Parser::new(tokens);
                let program = parser.parse_program();
                if let Some(e) = program.errors.into_iter().next() {
                    return Err(format!("Error parser di '{}': {:?}", path_str, e));
                }

                let new_base_path = final_path.parent().map(|p| p.to_path_buf());
                let mut fn_compiler = Compiler::baru_dengan_file(
                    self.heap,
                    new_base_path,
                    Some(final_path.to_string_lossy().to_string()),
                );

                for stmt in program.statements {
                    fn_compiler.compile_statement(stmt)?;
                }

                let mut chunk = fn_compiler.chunk;
                if chunk.code.last() != Some(&(OpCode::Return as u8)) {
                    let const_idx = chunk.write_constant(Value::Kosong);
                    chunk.write_opcode(OpCode::LoadConst, lokasi);
                    chunk.write_u16(const_idx, lokasi);
                    chunk.write_opcode(OpCode::Return, lokasi);
                }

                let fungsi = crate::value::FungsiVM {
                    nama: path_str.clone(),
                    parameter: vec![],
                    chunk,
                    file: Some(final_path.to_string_lossy().to_string()),
                };
                let fungsi_idx = self.heap.alloc(crate::heap::HeapData::Fungsi(fungsi));
                let func_val = Value::Fungsi(fungsi_idx, 0); // Dummy env_id 0 at compile time
                let const_idx = self.chunk.write_constant(func_val);

                self.chunk.write_opcode(OpCode::LoadConst, lokasi);
                self.chunk.write_u16(const_idx, lokasi);
                self.chunk.write_opcode(OpCode::LoadModule, lokasi);
            }
        }
        Ok(())
    }

    fn emit_jump(&mut self, instruction: OpCode, lokasi: Lokasi) -> usize {
        self.chunk.write_opcode(instruction, lokasi);
        self.chunk.write(0xff, lokasi);
        self.chunk.write(0xff, lokasi);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let target = self.chunk.code.len();
        let bytes = (target as u16).to_be_bytes();
        self.chunk.code[offset] = bytes[0];
        self.chunk.code[offset + 1] = bytes[1];
    }

    fn patch_jump_to(&mut self, offset: usize, target: usize) {
        let bytes = (target as u16).to_be_bytes();
        self.chunk.code[offset] = bytes[0];
        self.chunk.code[offset + 1] = bytes[1];
    }
}

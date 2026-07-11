use crate::opcodes::OpCode;
use crate::value::{Value, FungsiVM};
use ast::{Program, Statement, Expression, InfixOperator, PrefixOperator};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn write_opcode(&mut self, op: OpCode) {
        self.code.push(op as u8);
    }

    pub fn write_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    pub fn write_u16(&mut self, val: u16) {
        let bytes = val.to_be_bytes();
        self.code.push(bytes[0]);
        self.code.push(bytes[1]);
    }
}

pub struct Compiler {
    pub chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
        }
    }

    pub fn compile(mut self, program: Program) -> Result<Chunk, String> {
        for stmt in program.statements {
            self.compile_statement(stmt)?;
        }
        self.chunk.write_opcode(OpCode::Return);
        Ok(self.chunk)
    }

    fn compile_statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
            }
            Statement::DeklarasiVariabel { nama, nilai, .. } | Statement::Assignment { nama, nilai, .. } => {
                self.compile_expression(nilai)?;
                let const_idx = self.chunk.write_constant(Value::String(Rc::new(nama)));
                self.chunk.write_opcode(OpCode::StoreVar);
                self.chunk.write_u16(const_idx);
            }
            Statement::Tampilkan { nilai, .. } => {
                for expr in nilai {
                    self.compile_expression(expr)?;
                    self.chunk.write_opcode(OpCode::Print);
                }
            }
            Statement::Jika { kondisi, konsekuensi, alternatif, .. } => {
                self.compile_expression(kondisi)?;
                
                let jump_if_false_offset = self.emit_jump(OpCode::JumpIfFalse);
                
                for stmt in konsekuensi {
                    self.compile_statement(stmt)?;
                }

                let jump_offset = self.emit_jump(OpCode::Jump);
                
                self.patch_jump(jump_if_false_offset);
                
                if let Some(alt) = alternatif {
                    for stmt in alt {
                        self.compile_statement(stmt)?;
                    }
                }
                
                self.patch_jump(jump_offset);
            }
            Statement::Selama { kondisi, body, .. } => {
                let loop_start = self.chunk.code.len();
                
                self.compile_expression(kondisi)?;
                
                let jump_if_false_offset = self.emit_jump(OpCode::JumpIfFalse);
                
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                // Jump back to start
                let jump_back = self.emit_jump(OpCode::Jump);
                self.patch_jump_to(jump_back, loop_start);
                
                // Patch end
                self.patch_jump(jump_if_false_offset);
            }
            Statement::DeklarasiFungsi { nama, parameter, body, .. } => {
                let mut fn_compiler = Compiler::new();
                for stmt in body {
                    fn_compiler.compile_statement(stmt)?;
                }
                
                // Ensure function always returns
                let mut chunk = fn_compiler.chunk;
                if chunk.code.last() != Some(&(OpCode::Return as u8)) {
                    let const_idx = chunk.write_constant(Value::Kosong);
                    chunk.write_opcode(OpCode::LoadConst);
                    chunk.write_u16(const_idx);
                    chunk.write_opcode(OpCode::Return);
                }
                
                let fungsi = FungsiVM {
                    nama: nama.clone(),
                    parameter,
                    chunk,
                };
                let const_idx = self.chunk.write_constant(Value::Fungsi(Rc::new(fungsi)));
                self.chunk.write_opcode(OpCode::LoadConst);
                self.chunk.write_u16(const_idx);

                let name_idx = self.chunk.write_constant(Value::String(Rc::new(nama)));
                self.chunk.write_opcode(OpCode::StoreVar);
                self.chunk.write_u16(name_idx);
            }
            Statement::Kembalikan { nilai, .. } => {
                if let Some(expr) = nilai {
                    self.compile_expression(expr)?;
                } else {
                    let const_idx = self.chunk.write_constant(Value::Kosong);
                    self.chunk.write_opcode(OpCode::LoadConst);
                    self.chunk.write_u16(const_idx);
                }
                self.chunk.write_opcode(OpCode::Return);
            }
            _ => return Err(format!("Statement tidak didukung di VM: {:?}", stmt)),
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: Expression) -> Result<(), String> {
        match expr {
            Expression::Angka(val, _) => {
                let const_idx = self.chunk.write_constant(Value::Angka(val));
                self.chunk.write_opcode(OpCode::LoadConst);
                self.chunk.write_u16(const_idx);
            }
            Expression::String(val, _) => {
                let const_idx = self.chunk.write_constant(Value::String(Rc::new(val)));
                self.chunk.write_opcode(OpCode::LoadConst);
                self.chunk.write_u16(const_idx);
            }
            Expression::Boolean(val, _) => {
                let const_idx = self.chunk.write_constant(Value::Boolean(val));
                self.chunk.write_opcode(OpCode::LoadConst);
                self.chunk.write_u16(const_idx);
            }
            Expression::Identifier(nama, _) => {
                let const_idx = self.chunk.write_constant(Value::String(Rc::new(nama)));
                self.chunk.write_opcode(OpCode::LoadVar);
                self.chunk.write_u16(const_idx);
            }
            Expression::Infix { kiri, operator, kanan, .. } => {
                self.compile_expression(*kiri)?;
                self.compile_expression(*kanan)?;
                match operator {
                    InfixOperator::Tambah => self.chunk.write_opcode(OpCode::Add),
                    InfixOperator::Kurang => self.chunk.write_opcode(OpCode::Subtract),
                    InfixOperator::Kali => self.chunk.write_opcode(OpCode::Multiply),
                    InfixOperator::Bagi => self.chunk.write_opcode(OpCode::Divide),
                    InfixOperator::Mod => self.chunk.write_opcode(OpCode::Modulus),
                    InfixOperator::LebihDari => self.chunk.write_opcode(OpCode::Greater),
                    InfixOperator::KurangDari => self.chunk.write_opcode(OpCode::Less),
                    InfixOperator::Minimal => self.chunk.write_opcode(OpCode::GreaterEqual),
                    InfixOperator::Maksimal => self.chunk.write_opcode(OpCode::LessEqual),
                    InfixOperator::SamaDengan => self.chunk.write_opcode(OpCode::Equal),
                    InfixOperator::TidakSamaDengan => self.chunk.write_opcode(OpCode::NotEqual),
                    InfixOperator::Dan => self.chunk.write_opcode(OpCode::And),
                    InfixOperator::Atau => self.chunk.write_opcode(OpCode::Or),
                }
            }
            Expression::Prefix { operator, kanan, .. } => {
                self.compile_expression(*kanan)?;
                match operator {
                    PrefixOperator::Minus => {
                        // Implement negate in future
                        return Err("Prefix Minus not yet supported in VM".to_string());
                    }
                    PrefixOperator::Bukan => self.chunk.write_opcode(OpCode::Not),
                }
            }
            Expression::Call { fungsi, argumen, .. } => {
                self.compile_expression(*fungsi)?;
                let arg_count = argumen.len();
                if arg_count > 255 {
                    return Err("Argumen maksimal 255".to_string());
                }
                for arg in argumen {
                    self.compile_expression(arg)?;
                }
                self.chunk.write_opcode(OpCode::Call);
                self.chunk.write(arg_count as u8);
            }
            Expression::Index { kiri, indeks, .. } => {
                self.compile_expression(*kiri)?;
                self.compile_expression(*indeks)?;
                self.chunk.write_opcode(OpCode::GetIndex);
            }
            Expression::Array { elemen, .. } => {
                let count = elemen.len();
                if count > u16::MAX as usize {
                    return Err("Elemen array maksimal 65535".to_string());
                }
                for el in elemen {
                    self.compile_expression(el)?;
                }
                self.chunk.write_opcode(OpCode::MakeArray);
                self.chunk.write_u16(count as u16);
            }
            Expression::Kamus { pasangan, .. } => {
                let count = pasangan.len();
                if count > u16::MAX as usize {
                    return Err("Elemen kamus maksimal 65535".to_string());
                }
                for (k, v) in pasangan {
                    self.compile_expression(k)?;
                    self.compile_expression(v)?;
                }
                self.chunk.write_opcode(OpCode::MakeKamus);
                self.chunk.write_u16(count as u16);
            }
            _ => return Err(format!("Ekspresi tidak didukung di VM: {:?}", expr)),
        }
        Ok(())
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.chunk.write_opcode(instruction);
        self.chunk.write(0xff);
        self.chunk.write(0xff);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let target = self.chunk.code.len();
        if target > u16::MAX as usize {
            // handle error
        }
        let bytes = (target as u16).to_be_bytes();
        self.chunk.code[offset] = bytes[0];
        self.chunk.code[offset + 1] = bytes[1];
    }
    
    fn patch_jump_to(&mut self, offset: usize, target: usize) {
        // Backwards jump requires different calculation or we can just make it absolute.
        // Let's use absolute jumps for simplicity.
        // Change Jump implementation to be absolute!
        let bytes = (target as u16).to_be_bytes();
        self.chunk.code[offset] = bytes[0];
        self.chunk.code[offset + 1] = bytes[1];
    }
}

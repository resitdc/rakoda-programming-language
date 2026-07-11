use crate::opcodes::OpCode;
use crate::value::{Value, FungsiVM};
use crate::compiler::Chunk;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct CallFrame {
    pub fungsi: Rc<FungsiVM>,
    pub ip: usize,
    pub stack_offset: usize,
}

impl CallFrame {
    fn read_byte(&mut self) -> u8 {
        let byte = self.fungsi.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_short(&mut self) -> u16 {
        let b1 = self.read_byte();
        let b2 = self.read_byte();
        u16::from_be_bytes([b1, b2])
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_short();
        self.fungsi.chunk.constants[index as usize].clone()
    }
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(64),
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
        }
    }

    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn execute(&mut self, chunk: Chunk) -> Result<(), String> {
        let main_fungsi = Rc::new(FungsiVM {
            nama: "main".to_string(),
            parameter: vec![],
            chunk,
        });

        self.frames.push(CallFrame {
            fungsi: main_fungsi,
            ip: 0,
            stack_offset: 0,
        });

        self.run()
    }

    fn run(&mut self) -> Result<(), String> {
        loop {
            let instruction = {
                let frame = self.frames.last_mut().unwrap();
                frame.read_byte()
            };

            let opcode = OpCode::from_u8(instruction)
                .ok_or_else(|| format!("Unknown opcode {}", instruction))?;

            match opcode {
                OpCode::Return => {
                    let result = self.stack.pop().unwrap_or(Value::Kosong);
                    
                    let frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        return Ok(());
                    }
                    
                    // Cleanup stack arguments
                    self.stack.truncate(frame.stack_offset);
                    
                    // Push result back to stack
                    self.stack.push(result);
                }
                OpCode::LoadConst => {
                    let constant = self.frames.last_mut().unwrap().read_constant();
                    self.stack.push(constant);
                }
                OpCode::LoadVar => {
                    let name_val = self.frames.last_mut().unwrap().read_constant();
                    if let Value::String(name) = name_val {
                        let val = self.globals.get(name.as_ref())
                            .cloned()
                            .ok_or_else(|| format!("Variabel '{}' belum dibuat.", name))?;
                        self.stack.push(val);
                    }
                }
                OpCode::StoreVar => {
                    let name_val = self.frames.last_mut().unwrap().read_constant();
                    if let Value::String(name) = name_val {
                        let val = self.stack.pop().unwrap_or(Value::Kosong);
                        self.globals.insert(name.to_string(), val);
                    }
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Angka(a_val), Value::Angka(b_val)) => self.stack.push(Value::Angka(a_val + b_val)),
                        (Value::String(a_val), Value::String(b_val)) => {
                            self.stack.push(Value::String(Rc::new(format!("{}{}", a_val, b_val))))
                        }
                        _ => return Err("Operan harus angka atau teks untuk dijumlahkan".to_string())
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Angka(a_val - b_val));
                    } else {
                        return Err("Operan harus angka untuk dikurangkan".to_string());
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Angka(a_val * b_val));
                    } else {
                        return Err("Operan harus angka untuk dikali".to_string());
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        if b_val == 0.0 {
                            return Err("Pembagian dengan nol".to_string());
                        }
                        self.stack.push(Value::Angka(a_val / b_val));
                    } else {
                        return Err("Operan harus angka untuk dibagi".to_string());
                    }
                }
                OpCode::Modulus => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Angka(a_val % b_val));
                    } else {
                        return Err("Operan harus angka untuk modulus".to_string());
                    }
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::NotEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a != b));
                }
                OpCode::Greater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Boolean(a_val > b_val));
                    }
                }
                OpCode::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Boolean(a_val < b_val));
                    }
                }
                OpCode::GreaterEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Boolean(a_val >= b_val));
                    }
                }
                OpCode::LessEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Boolean(a_val <= b_val));
                    }
                }
                OpCode::And => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(is_truthy(&a) && is_truthy(&b)));
                }
                OpCode::Or => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(is_truthy(&a) || is_truthy(&b)));
                }
                OpCode::Not => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(!is_truthy(&a)));
                }
                OpCode::Print => {
                    let a = self.stack.pop().unwrap();
                    println!("{}", a);
                }
                OpCode::JumpIfFalse => {
                    let offset = self.frames.last_mut().unwrap().read_short() as usize;
                    let peek = self.stack.last().unwrap_or(&Value::Kosong);
                    if !is_truthy(peek) {
                        self.frames.last_mut().unwrap().ip = offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.frames.last_mut().unwrap().read_short() as usize;
                    self.frames.last_mut().unwrap().ip = offset;
                }
                OpCode::Call => {
                    let arg_count = self.frames.last_mut().unwrap().read_byte() as usize;
                    let fungsi_val = self.stack[self.stack.len() - arg_count - 1].clone();
                    
                    match fungsi_val {
                        Value::Fungsi(fungsi) => {
                            if arg_count != fungsi.parameter.len() {
                                return Err(format!("Fungsi '{}' membutuhkan {} argumen, tetapi diberikan {}", fungsi.nama, fungsi.parameter.len(), arg_count));
                            }
                            
                            // Assign arguments to globals (simple workaround for MVP)
                            for i in 0..arg_count {
                                let arg_val = self.stack[self.stack.len() - arg_count + i].clone();
                                self.globals.insert(fungsi.parameter[i].clone(), arg_val);
                            }
                            
                            let stack_offset = self.stack.len() - arg_count - 1;
                            self.frames.push(CallFrame {
                                fungsi,
                                ip: 0,
                                stack_offset,
                            });
                        }
                        Value::FungsiBawaan(fungsi) => {
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                args.push(self.stack.pop().unwrap());
                            }
                            args.reverse();
                            
                            // Pop the function itself
                            self.stack.pop();
                            
                            let result = (fungsi.func)(args)?;
                            self.stack.push(result);
                        }
                        _ => return Err("Hanya fungsi yang dapat dipanggil.".to_string()),
                    }
                }
                OpCode::GetIndex => {
                    let index = self.stack.pop().unwrap();
                    let target = self.stack.pop().unwrap();
                    
                    match target {
                        Value::Kamus(kamus) => {
                            if let Value::String(key) = index {
                                let val = kamus.borrow().get(key.as_ref()).cloned().unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err("Indeks kamus harus berupa teks".to_string());
                            }
                        }
                        Value::Array(arr) => {
                            if let Value::Angka(idx) = index {
                                let i = idx as usize;
                                let val = arr.borrow().get(i).cloned().unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err("Indeks array harus berupa angka".to_string());
                            }
                        }
                        _ => return Err("Operasi index tidak didukung untuk tipe ini".to_string()),
                    }
                }
                OpCode::MakeArray => {
                    let count = self.frames.last_mut().unwrap().read_short() as usize;
                    let mut elements = Vec::with_capacity(count);
                    for _ in 0..count {
                        elements.push(self.stack.pop().unwrap());
                    }
                    elements.reverse();
                    self.stack.push(Value::Array(Rc::new(RefCell::new(elements))));
                }
                OpCode::MakeKamus => {
                    let count = self.frames.last_mut().unwrap().read_short() as usize;
                    let mut map = HashMap::with_capacity(count);
                    for _ in 0..count {
                        let v = self.stack.pop().unwrap();
                        let k = self.stack.pop().unwrap();
                        if let Value::String(key_str) = k {
                            map.insert(key_str.to_string(), v);
                        } else {
                            return Err("Kunci kamus harus berupa teks".to_string());
                        }
                    }
                    // Since we pop from top of stack, order doesn't strictly matter for HashMap,
                    // but we did pop Value then Key correctly because Key is compiled first, then Value.
                    self.stack.push(Value::Kamus(Rc::new(RefCell::new(map))));
                }
            }
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Kosong => false,
        Value::Boolean(b) => *b,
        Value::Angka(a) => *a != 0.0,
        _ => true,
    }
}

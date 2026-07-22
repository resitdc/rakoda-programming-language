use crate::compiler::Chunk;
use crate::heap::{Heap, HeapData};
use crate::opcodes::OpCode;
use crate::value::{FungsiVM, Value};
use errors::Lokasi;
use std::collections::HashMap;

type TaskResult = (Result<crate::value::Value, String>, crate::heap::Heap);

#[derive(Clone)]
pub struct CallFrame {
    pub fungsi: usize, // index to Heap.fungsi
    pub ip: usize,
    pub stack_offset: usize,
    pub env_id: usize,
    pub is_module: bool,
}

impl CallFrame {
    pub fn read_byte(&mut self, heap: &Heap) -> u8 {
        let byte = heap.get_fungsi(self.fungsi).chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    pub fn read_short(&mut self, heap: &Heap) -> u16 {
        let b1 = self.read_byte(heap) as u16;
        let b2 = self.read_byte(heap) as u16;
        (b1 << 8) | b2
    }

    fn read_constant(&mut self, heap: &Heap) -> Value {
        let index = self.read_short(heap);
        heap.get_fungsi(self.fungsi).chunk.constants[index as usize]
    }
}

#[derive(Clone)]
pub struct CatchHandler {
    pub frame_index: usize,
    pub stack_offset: usize,
    pub ip_offset: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    pub environments: Vec<HashMap<String, Value>>,
    pub modules_cache: HashMap<String, Value>,
    pub heap: Heap,
    pub tasks: HashMap<usize, std::thread::JoinHandle<TaskResult>>,
    pub next_task_id: usize,
    pub catch_handlers: Vec<CatchHandler>,
    pub next_gc_threshold: usize,
    pub capture_output: bool,
    pub output_buffer: String,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(64),
            stack: Vec::with_capacity(256),
            environments: vec![HashMap::new()],
            modules_cache: HashMap::new(),
            heap: Heap::new(),
            tasks: HashMap::new(),
            next_task_id: 1,
            catch_handlers: Vec::new(),
            next_gc_threshold: 1000,
            capture_output: false,
            output_buffer: String::new(),
        }
    }

    pub fn clone_vm(&self) -> VM {
        VM {
            frames: self.frames.clone(),
            stack: self.stack.clone(),
            environments: self.environments.clone(),
            modules_cache: self.modules_cache.clone(),
            heap: self.heap.clone(),
            tasks: HashMap::new(),
            next_task_id: 1,
            catch_handlers: self.catch_handlers.clone(),
            next_gc_threshold: 1000,
            capture_output: false,
            output_buffer: String::new(),
        }
    }

    pub fn set_global(&mut self, name: String, value: Value) {
        self.environments[0].insert(name, value);
    }

    pub fn gc_collect(&mut self) {
        let mut roots = Vec::new();
        for val in &self.stack {
            roots.push(*val);
        }
        for env in &self.environments {
            for val in env.values() {
                roots.push(*val);
            }
        }
        for val in self.modules_cache.values() {
            roots.push(*val);
        }
        for frame in &self.frames {
            roots.push(Value::Fungsi(frame.fungsi, frame.env_id));
        }

        for val in roots {
            match val {
                Value::Array(i)
                | Value::Kamus(i)
                | Value::String(i)
                | Value::Fungsi(i, _)
                | Value::FungsiBawaan(i)
                | Value::Modul(i) => {
                    self.heap.mark(i);
                }
                _ => {}
            }
        }

        self.heap.mark_sessions_and_cache();

        let before = self.heap.allocated_count;
        self.heap.sweep();
        let after = self.heap.allocated_count;

        if before > after {
            // println!("[GC] Dibersihkan: {} objek", before - after); // Disabled to keep output clean, but can be enabled for debugging
        }

        self.next_gc_threshold = std::cmp::max(1000, self.heap.allocated_count * 2);
    }

    fn current_lokasi(&self) -> Option<Lokasi> {
        if let Some(frame) = self.frames.last() {
            let ip = if frame.ip > 0 { frame.ip - 1 } else { 0 };
            if let crate::heap::HeapData::Fungsi(f) = &self.heap.objects[frame.fungsi].data {
                return f.chunk.locations.get(ip).copied();
            }
        }
        None
    }

    fn err(&self, msg: impl Into<String>) -> (String, Option<Lokasi>) {
        (msg.into(), self.current_lokasi())
    }

    pub fn execute(&mut self, chunk: Chunk) -> Result<(), (String, Option<Lokasi>)> {
        let main_fungsi = FungsiVM {
            nama: "main".to_string(),
            parameter: vec![],
            chunk,
            file: None,
        };
        let fungsi_idx = self.heap.alloc(HeapData::Fungsi(main_fungsi));

        let stack_offset = self.stack.len();
        let initial_frames = self.frames.len();

        self.frames.push(CallFrame {
            fungsi: fungsi_idx,
            ip: 0,
            stack_offset,
            env_id: 0,
            is_module: false,
        });

        self.run(initial_frames)
    }

    fn run(&mut self, initial_frames: usize) -> Result<(), (String, Option<Lokasi>)> {
        loop {
            // Trigger GC if we allocated a lot
            if self.heap.allocated_count > self.next_gc_threshold {
                self.gc_collect();
            }

            let instruction = {
                let frame = self.frames.last_mut().unwrap();
                frame.read_byte(&self.heap)
            };

            let opcode = OpCode::from_u8(instruction)
                .ok_or_else(|| self.err(format!("Unknown opcode {}", instruction)))?;

            match opcode {
                OpCode::Return => {
                    let result = self.stack.pop().unwrap_or(Value::Kosong);

                    let frame = self.frames.pop().unwrap();

                    self.stack.truncate(frame.stack_offset);

                    if frame.is_module {
                        let env = self.environments[frame.env_id].clone();
                        let modul_idx = self.heap.alloc(HeapData::Modul(env));
                        let modul_val = Value::Modul(modul_idx);

                        let path_str = self.heap.get_fungsi(frame.fungsi).nama.clone();
                        self.modules_cache.insert(path_str, modul_val);

                        self.stack.push(modul_val);
                    } else {
                        self.stack.push(result);
                    }

                    if self.frames.len() == initial_frames {
                        return Ok(());
                    }
                }
                OpCode::LoadConst => {
                    let mut constant = self.frames.last_mut().unwrap().read_constant(&self.heap);
                    if let Value::Fungsi(idx, _) = constant {
                        constant = Value::Fungsi(idx, self.frames.last().unwrap().env_id);
                    }
                    self.stack.push(constant);
                }
                OpCode::LoadVar => {
                    let (name_idx_val, env_id) = {
                        let frame = self.frames.last_mut().unwrap();
                        (frame.read_constant(&self.heap), frame.env_id)
                    };
                    if let Value::String(name_idx) = name_idx_val {
                        let name = self.heap.get_string(name_idx).clone();
                        let val = if let Some(v) = self.environments[env_id].get(&name) {
                            Some(*v)
                        } else if env_id != 0 {
                            self.environments[0].get(&name).cloned()
                        } else {
                            None
                        };
                        let val = val.ok_or_else(|| {
                            self.err(format!("Variabel '{}' belum dibuat.", name))
                        })?;
                        self.stack.push(val);
                    }
                }
                OpCode::StoreVar => {
                    let (name_idx_val, env_id) = {
                        let frame = self.frames.last_mut().unwrap();
                        (frame.read_constant(&self.heap), frame.env_id)
                    };
                    if let Value::String(name_idx) = name_idx_val {
                        let name = self.heap.get_string(name_idx).clone();
                        let val = self.stack.pop().unwrap_or(Value::Kosong);
                        self.environments[env_id].insert(name, val);
                    }
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Angka(a_val), Value::Angka(b_val)) => {
                            self.stack.push(Value::Angka(a_val + b_val))
                        }
                        (a, b) => {
                            let is_a_string = matches!(a, Value::String(_));
                            let is_b_string = matches!(b, Value::String(_));
                            if is_a_string || is_b_string {
                                let s1 = a.to_string(&self.heap);
                                let s2 = b.to_string(&self.heap);
                                let new_idx =
                                    self.heap.alloc(HeapData::String(format!("{}{}", s1, s2)));
                                self.stack.push(Value::String(new_idx));
                            } else {
                                return Err(
                                    self.err("Operan harus angka atau teks untuk dijumlahkan")
                                );
                            }
                        }
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Angka(a_val - b_val));
                    } else {
                        return Err(self.err("Operan harus angka untuk dikurangkan"));
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Angka(a_val * b_val));
                    } else {
                        return Err(self.err("Operan harus angka untuk dikali"));
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        if b_val == 0.0 {
                            return Err(self.err("Pembagian dengan nol"));
                        }
                        self.stack.push(Value::Angka(a_val / b_val));
                    } else {
                        return Err(self.err("Operan harus angka untuk dibagi"));
                    }
                }
                OpCode::Modulus => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Angka(a_val), Value::Angka(b_val)) = (a, b) {
                        self.stack.push(Value::Angka(a_val % b_val));
                    } else {
                        return Err(self.err("Operan harus angka untuk modulus"));
                    }
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if let (Value::String(a_idx), Value::String(b_idx)) = (a, b) {
                        let a_str = self.heap.get_string(a_idx).clone();
                        let b_str = self.heap.get_string(b_idx).clone();
                        self.stack.push(Value::Boolean(a_str == b_str));
                    } else {
                        self.stack.push(Value::Boolean(a == b));
                    }
                }
                OpCode::NotEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if let (Value::String(a_idx), Value::String(b_idx)) = (a, b) {
                        let a_str = self.heap.get_string(a_idx).clone();
                        let b_str = self.heap.get_string(b_idx).clone();
                        self.stack.push(Value::Boolean(a_str != b_str));
                    } else {
                        self.stack.push(Value::Boolean(a != b));
                    }
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
                    self.stack
                        .push(Value::Boolean(is_truthy(&a) && is_truthy(&b)));
                }
                OpCode::Or => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack
                        .push(Value::Boolean(is_truthy(&a) || is_truthy(&b)));
                }
                OpCode::Not => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(!is_truthy(&a)));
                }
                OpCode::Print => {
                    let a = self.stack.pop().unwrap();
                    let s = a.to_string(&self.heap);
                    if self.capture_output {
                        self.output_buffer.push_str(&s);
                        self.output_buffer.push('\n');
                    } else {
                        println!("{}", s);
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.frames.last_mut().unwrap().read_short(&self.heap) as usize;
                    let peek = self.stack.last().unwrap_or(&Value::Kosong);
                    if !is_truthy(peek) {
                        self.frames.last_mut().unwrap().ip = offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.frames.last_mut().unwrap().read_short(&self.heap) as usize;
                    self.frames.last_mut().unwrap().ip = offset;
                }
                OpCode::Call => {
                    let arg_count = self.frames.last_mut().unwrap().read_byte(&self.heap) as usize;
                    let fungsi_val = self.stack[self.stack.len() - arg_count - 1];

                    match fungsi_val {
                        Value::Fungsi(fungsi_idx, env_id) => {
                            let (p_len, params) = {
                                let f = self.heap.get_fungsi(fungsi_idx);
                                (f.parameter.len(), f.parameter.clone())
                            };
                            if arg_count != p_len {
                                let f_nama = self.heap.get_fungsi(fungsi_idx).nama.clone();
                                return Err(self.err(format!(
                                    "Fungsi '{}' membutuhkan {} argumen, tetapi diberikan {}",
                                    f_nama, p_len, arg_count
                                )));
                            }

                            for (i, param) in params.iter().enumerate().take(arg_count) {
                                let arg_val = self.stack[self.stack.len() - arg_count + i];
                                self.environments[env_id].insert(param.clone(), arg_val);
                            }

                            let stack_offset = self.stack.len() - arg_count - 1;
                            self.frames.push(CallFrame {
                                fungsi: fungsi_idx,
                                ip: 0,
                                stack_offset,
                                env_id,
                                is_module: false,
                            });
                        }
                        Value::FungsiBawaan(fungsi_idx) => {
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                args.push(self.stack.pop().unwrap());
                            }
                            args.reverse();

                            self.stack.pop(); // Pop function itself

                            let func_ptr = self.heap.get_fungsi_bawaan(fungsi_idx).func.clone();
                            let result = match func_ptr(self, args) {
                                Ok(val) => val,
                                Err(e) => {
                                    if let Some(handler) = self.catch_handlers.pop() {
                                        self.frames.truncate(handler.frame_index);
                                        self.stack.truncate(handler.stack_offset);
                                        self.frames.last_mut().unwrap().ip = handler.ip_offset;
                                        let err_idx = self.heap.alloc(crate::heap::HeapData::String(e));
                                        self.stack.push(Value::String(err_idx));
                                        continue;
                                    } else {
                                        return Err(self.err(e));
                                    }
                                }
                            };
                            self.stack.push(result);
                        }
                        _ => return Err(self.err("Hanya fungsi yang dapat dipanggil".to_string())),
                    }
                }
                OpCode::Negate => {
                    let a = self.stack.pop().unwrap();
                    if let Value::Angka(a_val) = a {
                        self.stack.push(Value::Angka(-a_val));
                    } else {
                        return Err(self.err("Prefix minus hanya dapat digunakan pada angka"));
                    }
                }
                OpCode::LoadModule => {
                    let path_idx_val = self.stack.pop().unwrap();
                    let (path_str, func_idx) = if let Value::Fungsi(idx, _) = path_idx_val {
                        let path = self.heap.get_fungsi(idx).nama.clone();
                        (path, idx)
                    } else {
                        return Err(self.err("LoadModule requires a Fungsi constant".to_string()));
                    };

                    let is_html_template = path_str.ends_with(".rpl.html");
                    if !is_html_template && let Some(modul_val) = self.modules_cache.get(&path_str)
                    {
                        self.stack.push(*modul_val);
                        continue;
                    }

                    let env_id = if is_html_template {
                        self.frames.last().unwrap().env_id
                    } else {
                        let new_env_id = self.environments.len();
                        self.environments.push(HashMap::new());
                        new_env_id
                    };

                    let stack_offset = self.stack.len();
                    self.frames.push(CallFrame {
                        fungsi: func_idx,
                        ip: 0,
                        stack_offset,
                        env_id,
                        is_module: !is_html_template,
                    });
                }
                OpCode::GetIndex => {
                    let index = self.stack.pop().unwrap();
                    let target = self.stack.pop().unwrap();

                    match target {
                        Value::Kamus(k_idx) => {
                            if let Value::String(key_idx) = index {
                                let key_str = self.heap.get_string(key_idx).clone();
                                let val = self
                                    .heap
                                    .get_kamus(k_idx)
                                    .get(&key_str)
                                    .cloned()
                                    .unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err(self.err("Indeks kamus harus berupa teks"));
                            }
                        }
                        Value::Array(a_idx) => {
                            if let Value::Angka(idx) = index {
                                let i = idx as usize;
                                let val = self
                                    .heap
                                    .get_array(a_idx)
                                    .get(i)
                                    .cloned()
                                    .unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err(self.err("Indeks array harus berupa angka"));
                            }
                        }
                        Value::Modul(m_idx) => {
                            if let Value::String(key_idx) = index {
                                let key_str = self.heap.get_string(key_idx).clone();
                                let val = self
                                    .heap
                                    .get_modul(m_idx)
                                    .get(&key_str)
                                    .cloned()
                                    .unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err(self.err("Indeks modul harus berupa teks"));
                            }
                        }
                        _ => return Err(self.err("Operasi index tidak didukung untuk tipe ini")),
                    }
                }
                OpCode::MakeArray => {
                    let count = self.frames.last_mut().unwrap().read_short(&self.heap) as usize;
                    let mut elements = Vec::with_capacity(count);
                    for _ in 0..count {
                        elements.push(self.stack.pop().unwrap());
                    }
                    elements.reverse();
                    let new_idx = self.heap.alloc(HeapData::Array(elements));
                    self.stack.push(Value::Array(new_idx));
                }
                OpCode::MakeKamus => {
                    let count = self.frames.last_mut().unwrap().read_short(&self.heap) as usize;
                    let mut map = HashMap::with_capacity(count);
                    for _ in 0..count {
                        let v = self.stack.pop().unwrap();
                        let k = self.stack.pop().unwrap();
                        if let Value::String(key_idx) = k {
                            let key_str = self.heap.get_string(key_idx).clone();
                            map.insert(key_str, v);
                        } else {
                            return Err(self.err("Kunci kamus harus berupa teks"));
                        }
                    }
                    let new_idx = self.heap.alloc(HeapData::Kamus(map));
                    self.stack.push(Value::Kamus(new_idx));
                }
                OpCode::IterInit => {
                    let koleksi = self.stack.last().unwrap_or(&Value::Kosong);
                    match koleksi {
                        Value::Array(_) | Value::String(_) | Value::Kamus(_) => {
                            self.stack.push(Value::Angka(0.0));
                        }
                        _ => return Err(self.err("Hanya tipe daftar (array), teks (string), atau kamus yang bisa diulang dengan 'setiap'.")),
                    }
                }
                OpCode::IterNext => {
                    let offset = self.frames.last_mut().unwrap().read_short(&self.heap) as usize;
                    let counter_val = self.stack.last().unwrap().clone();
                    let koleksi_val = self.stack[self.stack.len() - 2].clone();
                    
                    let counter = if let Value::Angka(n) = counter_val { n as usize } else { 0 };
                    
                    let mut has_next = false;
                    let mut current_idx = Value::Kosong;
                    let mut current_val = Value::Kosong;
                    
                    match koleksi_val {
                        Value::Array(idx) => {
                            let arr = self.heap.get_array(idx);
                            if counter < arr.len() {
                                has_next = true;
                                current_idx = Value::Angka(counter as f64);
                                current_val = arr[counter].clone();
                            }
                        }
                        Value::String(idx) => {
                            let (has_n, char_s) = {
                                let s = self.heap.get_string(idx);
                                let chars: Vec<char> = s.chars().collect();
                                if counter < chars.len() {
                                    (true, Some(chars[counter].to_string()))
                                } else {
                                    (false, None)
                                }
                            };
                            if has_n {
                                has_next = true;
                                current_idx = Value::Angka(counter as f64);
                                let char_idx = self.heap.alloc(crate::heap::HeapData::String(char_s.unwrap()));
                                current_val = Value::String(char_idx);
                            }
                        }
                        Value::Kamus(idx) => {
                            let (has_n, key_str, val) = {
                                let k = self.heap.get_kamus(idx);
                                let mut keys: Vec<&String> = k.keys().collect();
                                keys.sort();
                                if counter < keys.len() {
                                    let key_s = keys[counter].clone();
                                    let val_c = k[&key_s].clone();
                                    (true, Some(key_s), Some(val_c))
                                } else {
                                    (false, None, None)
                                }
                            };
                            if has_n {
                                has_next = true;
                                let key_idx = self.heap.alloc(crate::heap::HeapData::String(key_str.unwrap()));
                                current_idx = Value::String(key_idx);
                                current_val = val.unwrap();
                            }
                        }
                        _ => {}
                    }
                    
                    if has_next {
                        let top_idx = self.stack.len() - 1;
                        self.stack[top_idx] = Value::Angka((counter + 1) as f64);
                        
                        self.stack.push(current_idx);
                        self.stack.push(current_val);
                    } else {
                        self.frames.last_mut().unwrap().ip = offset;
                    }
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::SetupCatch => {
                    let offset = self.frames.last_mut().unwrap().read_short(&self.heap) as usize;
                    self.catch_handlers.push(CatchHandler {
                        frame_index: self.frames.len(),
                        stack_offset: self.stack.len(),
                        ip_offset: offset,
                    });
                }
                OpCode::PopCatch => {
                    self.catch_handlers.pop();
                }
                OpCode::Throw => {
                    let error_val = self.stack.pop().unwrap();
                    if let Some(handler) = self.catch_handlers.pop() {
                        self.frames.truncate(handler.frame_index);
                        self.stack.truncate(handler.stack_offset);
                        self.frames.last_mut().unwrap().ip = handler.ip_offset;
                        self.stack.push(error_val);
                    } else {
                        return Err(self.err(format!(
                            "Unhandled exception: {}",
                            error_val.to_string(&self.heap)
                        )));
                    }
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

use crate::value::VmContext;

impl VmContext for VM {
    fn get_heap_mut(&mut self) -> &mut Heap {
        &mut self.heap
    }

    fn compile_source(&mut self, source: &str) -> Result<Value, String> {
        let mut lexer = lexer::Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| format!("{:?}", e))?;

        let mut parser = parser::Parser::new(tokens);
        let mut program = parser.parse_program();
        let errors = std::mem::take(&mut program.errors);
        if let Some(e) = errors.into_iter().next() {
            return Err(format!("{:?}", e));
        }

        // We create a new chunk by compiling the program
        let compiler = crate::compiler::Compiler::baru_dengan_base_path(&mut self.heap, None);
        let chunk = compiler.compile(program)?;

        let start_stack = self.stack.len();
        self.execute(chunk).map_err(|(e, _)| e)?;

        if self.stack.len() > start_stack {
            Ok(self.stack.pop().unwrap())
        } else {
            Ok(Value::Kosong)
        }
    }

    fn execute_function(&mut self, func_val: Value, args: Vec<Value>) -> Result<Value, String> {
        match func_val {
            Value::Fungsi(func_idx, env_id) => {
                let func = self.heap.get_fungsi(func_idx).clone();

                // Push args
                for arg in &args {
                    self.stack.push(*arg);
                }

                // Insert into environment
                for i in 0..func.parameter.len() {
                    if i < args.len() {
                        self.environments[env_id].insert(func.parameter[i].clone(), args[i]);
                    }
                }

                let stack_offset = self.stack.len() - args.len();
                self.frames.push(CallFrame {
                    fungsi: func_idx,
                    ip: 0,
                    stack_offset,
                    env_id,
                    is_module: false,
                });

                let target_frames = self.frames.len() - 1;
                match self.run(target_frames) {
                    Ok(_) => {
                        let result = self.stack.pop().unwrap_or(Value::Kosong);
                        Ok(result)
                    }
                    Err((msg, lokasi)) => {
                        let mut error_msg = msg.clone();
                        if let Some(loc) = lokasi {
                            let (fn_name, fn_file) = self.current_function_info();
                            let file_str = fn_file
                                .map(|f| format!("di file '{}', ", f))
                                .unwrap_or_default();
                            error_msg = format!(
                                "{} ({}fungsi '{}', baris {}, kolom {})",
                                msg, file_str, fn_name, loc.baris, loc.kolom
                            );

                            if msg.contains("Hanya fungsi yang dapat dipanggil")
                                || msg.contains("Bukan fungsi")
                            {
                                error_msg.push_str("\n\nSaran: Anda mencoba memanggil nilai yang bukan fungsi (misalnya Kosong/null). Ini sering terjadi akibat:\n1. Typo pada nama method/variabel (contoh: log.infos seharusnya log.info).\n2. Tidak menggunakan 'kembalikan' di akhir controller sebelum pemanggilan render.");
                            }
                        }
                        Err(error_msg)
                    }
                }
            }
            Value::FungsiBawaan(idx) => {
                let func_ptr = self.heap.get_fungsi_bawaan(idx).func.clone();
                func_ptr(self, args)
            }
            _ => Err("Bukan fungsi".to_string()),
        }
    }

    fn spawn_task(&mut self, func_val: Value) -> Result<usize, String> {
        let mut vm_clone = self.clone_vm();

        let handle = std::thread::spawn(move || {
            let res = vm_clone.execute_function(func_val, vec![]);
            (res, vm_clone.heap)
        });

        let task_id = self.next_task_id;
        self.next_task_id += 1;
        self.tasks.insert(task_id, handle);

        Ok(task_id)
    }

    fn join_task(&mut self, task_id: usize) -> Result<Value, String> {
        if let Some(handle) = self.tasks.remove(&task_id) {
            match handle.join() {
                Ok((res, background_heap)) => match res {
                    Ok(val) => {
                        let copied_val =
                            crate::value::deep_copy_value(&val, &background_heap, &mut self.heap);
                        Ok(copied_val)
                    }
                    Err(e) => Err(e),
                },
                Err(_) => Err("Gagal menunggu tugas background (Thread Panicked)".to_string()),
            }
        } else {
            Err(format!(
                "Tiket tugas dengan ID {} tidak ditemukan.",
                task_id
            ))
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn current_lokasi(&self) -> Option<errors::Lokasi> {
        self.current_lokasi()
    }

    fn current_function_info(&self) -> (String, Option<String>) {
        if let Some(frame) = self.frames.last()
            && let crate::heap::HeapData::Fungsi(f) = &self.heap.objects[frame.fungsi].data
        {
            return (f.nama.clone(), f.file.clone());
        }
        ("utama".to_string(), None)
    }
}

use crate::opcodes::OpCode;
use crate::compiler::Chunk;
use std::collections::HashMap;
use crate::heap::{Heap, HeapData};
use crate::value::{FungsiVM, Value};
use errors::Lokasi;

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
    pub tasks: HashMap<usize, std::thread::JoinHandle<(Result<Value, String>, Heap)>>,
    pub next_task_id: usize,
    pub catch_handlers: Vec<CatchHandler>,
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
        }
    }

    pub fn set_global(&mut self, name: String, value: Value) {
        self.environments[0].insert(name, value);
    }

    pub fn gc_collect(&mut self) {
        let mut roots = Vec::new();
        for val in &self.stack { roots.push(*val); }
        for env in &self.environments {
            for val in env.values() { roots.push(*val); }
        }
        for val in self.modules_cache.values() { roots.push(*val); }
        for frame in &self.frames { roots.push(Value::Fungsi(frame.fungsi, frame.env_id)); }
        
        for val in roots {
            match val {
                Value::Array(i) | Value::Kamus(i) | Value::String(i) | Value::Fungsi(i, _) | Value::FungsiBawaan(i) | Value::Modul(i) => {
                    self.heap.mark(i);
                },
                _ => {}
            }
        }
        
        self.heap.mark_sessions();
        
        let before = self.heap.allocated_count;
        self.heap.sweep();
        let after = self.heap.allocated_count;
        
        if before > after {
            // println!("[GC] Dibersihkan: {} objek", before - after); // Disabled to keep output clean, but can be enabled for debugging
        }
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
        };
        let fungsi_idx = self.heap.alloc(HeapData::Fungsi(main_fungsi));

        self.frames.push(CallFrame {
            fungsi: fungsi_idx,
            ip: 0,
            stack_offset: 0,
            env_id: 0,
            is_module: false,
        });

        self.run(0)
    }

    fn run(&mut self, initial_frames: usize) -> Result<(), (String, Option<Lokasi>)> {
        loop {
            // Trigger GC if we allocated a lot
            if self.heap.allocated_count > 1000 {
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
                        let val = self.environments[env_id].get(&name)
                            .cloned()
                            .ok_or_else(|| self.err(format!("Variabel '{}' belum dibuat.", name)))?;
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
                        (Value::Angka(a_val), Value::Angka(b_val)) => self.stack.push(Value::Angka(a_val + b_val)),
                        (a, b) => {
                            let is_a_string = matches!(a, Value::String(_));
                            let is_b_string = matches!(b, Value::String(_));
                            if is_a_string || is_b_string {
                                let s1 = a.to_string(&self.heap);
                                let s2 = b.to_string(&self.heap);
                                let new_idx = self.heap.alloc(HeapData::String(format!("{}{}", s1, s2)));
                                self.stack.push(Value::String(new_idx));
                            } else {
                                return Err(self.err("Operan harus angka atau teks untuk dijumlahkan"));
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
                    println!("{}", a.to_string(&self.heap));
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
                                return Err(self.err(format!("Fungsi '{}' membutuhkan {} argumen, tetapi diberikan {}", f_nama, p_len, arg_count)));
                            }
                            
                            for i in 0..arg_count {
                                let arg_val = self.stack[self.stack.len() - arg_count + i];
                                self.environments[env_id].insert(params[i].clone(), arg_val);
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
                            
                            let func_ptr = self.heap.get_fungsi_bawaan(fungsi_idx).func;
                            // Pass heap implicitly
                            let result = func_ptr(self, args).map_err(|e| self.err(e))?;
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
                    if !is_html_template {
                        if let Some(modul_val) = self.modules_cache.get(&path_str) {
                            self.stack.push(*modul_val);
                            continue;
                        }
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
                                let val = self.heap.get_kamus(k_idx).get(&key_str).cloned().unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err(self.err("Indeks kamus harus berupa teks"));
                            }
                        }
                        Value::Array(a_idx) => {
                            if let Value::Angka(idx) = index {
                                let i = idx as usize;
                                let val = self.heap.get_array(a_idx).get(i).cloned().unwrap_or(Value::Kosong);
                                self.stack.push(val);
                            } else {
                                return Err(self.err("Indeks array harus berupa angka"));
                            }
                        }
                        Value::Modul(m_idx) => {
                            if let Value::String(key_idx) = index {
                                let key_str = self.heap.get_string(key_idx).clone();
                                let val = self.heap.get_modul(m_idx).get(&key_str).cloned().unwrap_or(Value::Kosong);
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
                        return Err(self.err(format!("Unhandled exception: {}", error_val.to_string(&self.heap))));
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

    fn execute_function(&mut self, func_val: Value, args: Vec<Value>) -> Result<Value, String> {
        let (func_idx, env_id) = if let Value::Fungsi(idx, env) = func_val {
            (idx, env)
        } else {
            return Err("Bukan fungsi".to_string());
        };

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
            Err((msg, _lokasi)) => Err(msg),
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
                Ok((res, background_heap)) => {
                    match res {
                        Ok(val) => {
                            let copied_val = crate::value::deep_copy_value(&val, &background_heap, &mut self.heap);
                            Ok(copied_val)
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(_) => Err("Gagal menunggu tugas background (Thread Panicked)".to_string()),
            }
        } else {
            Err(format!("Tiket tugas dengan ID {} tidak ditemukan.", task_id))
        }
    }
}

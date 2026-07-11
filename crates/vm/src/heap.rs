use std::collections::HashMap;
use crate::value::{Value, FungsiVM, FungsiBawaanVM};

#[derive(Clone)]
pub struct HeapObject {
    pub is_marked: bool,
    pub data: HeapData,
}

#[derive(Clone)]
pub enum HeapData {
    String(String),
    Array(Vec<Value>),
    Kamus(HashMap<String, Value>),
    Fungsi(FungsiVM),
    FungsiBawaan(FungsiBawaanVM),
    Modul(HashMap<String, Value>),
    Free(usize), // Next free index
}

#[derive(Clone)]
pub struct WebConfig {
    pub kompresi: bool,
    pub rate_limit: Option<u32>,
    pub proxies: HashMap<String, String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            kompresi: false,
            rate_limit: None,
            proxies: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct WebState {
    // SessionID -> (Expired, Data)
    // Value represents Kamus data in memory but wait, Value itself is an index.
    // So session data needs to be stored somewhere. We can store it as a Kamus inside WebState itself instead of in the VM Heap?
    // Actually, storing it in the VM Heap is fine, but since we are modifying WebState across requests, we can just store `HashMap<String, usize>` where usize is the Kamus Heap index.
    pub sessions: HashMap<String, (Option<std::time::Instant>, usize)>,
    
    pub active_session_id: Option<String>,
    pub active_cookies: HashMap<String, String>,
    pub cookies_to_set: Vec<String>,
}

impl Default for WebState {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            active_cookies: HashMap::new(),
            cookies_to_set: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Heap {
    pub objects: Vec<HeapObject>,
    pub free_list_head: Option<usize>,
    pub allocated_count: usize,
    pub web_routes: HashMap<String, Value>,
    pub web_config: WebConfig,
    pub web_state: WebState,
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            free_list_head: None,
            allocated_count: 0,
            web_routes: HashMap::new(),
            web_config: WebConfig::default(),
            web_state: WebState::default(),
        }
    }

    pub fn alloc(&mut self, data: HeapData) -> usize {
        self.allocated_count += 1;
        
        if let Some(idx) = self.free_list_head
            && let HeapData::Free(next) = self.objects[idx].data {
                self.free_list_head = if next == usize::MAX { None } else { Some(next) };
                self.objects[idx] = HeapObject {
                    is_marked: false,
                    data,
                };
                return idx;
            }
        
        let idx = self.objects.len();
        self.objects.push(HeapObject {
            is_marked: false,
            data,
        });
        idx
    }

    pub fn free(&mut self, idx: usize) {
        let next = self.free_list_head.unwrap_or(usize::MAX);
        self.objects[idx].data = HeapData::Free(next);
        self.free_list_head = Some(idx);
        self.allocated_count -= 1;
    }

    pub fn get_string(&self, idx: usize) -> &String {
        if let HeapData::String(s) = &self.objects[idx].data {
            s
        } else {
            panic!("Expected String at heap index {}", idx);
        }
    }

    pub fn get_array(&self, idx: usize) -> &Vec<Value> {
        if let HeapData::Array(arr) = &self.objects[idx].data {
            arr
        } else {
            panic!("Expected Array at heap index {}", idx);
        }
    }

    pub fn get_array_mut(&mut self, idx: usize) -> &mut Vec<Value> {
        if let HeapData::Array(arr) = &mut self.objects[idx].data {
            arr
        } else {
            panic!("Expected Array at heap index {}", idx);
        }
    }

    pub fn get_kamus(&self, idx: usize) -> &HashMap<String, Value> {
        if let HeapData::Kamus(k) = &self.objects[idx].data {
            k
        } else {
            panic!("Expected Kamus at heap index {}", idx);
        }
    }

    pub fn get_kamus_mut(&mut self, idx: usize) -> &mut HashMap<String, Value> {
        if let HeapData::Kamus(k) = &mut self.objects[idx].data {
            k
        } else {
            panic!("Expected Kamus at heap index {}", idx);
        }
    }

    pub fn get_modul(&self, idx: usize) -> &HashMap<String, Value> {
        if let HeapData::Modul(m) = &self.objects[idx].data {
            m
        } else {
            panic!("Expected Modul at heap index {}", idx);
        }
    }

    pub fn get_fungsi(&self, idx: usize) -> &FungsiVM {
        if let HeapData::Fungsi(f) = &self.objects[idx].data {
            f
        } else {
            panic!("Expected Fungsi at heap index {}", idx);
        }
    }

    pub fn get_fungsi_bawaan(&self, idx: usize) -> &FungsiBawaanVM {
        if let HeapData::FungsiBawaan(f) = &self.objects[idx].data {
            f
        } else {
            panic!("Expected FungsiBawaan at heap index {}", idx);
        }
    }



    pub fn mark(&mut self, idx: usize) {
        if self.objects[idx].is_marked { return; }
        self.objects[idx].is_marked = true;

        // Recursively mark children
        let mut worklist = vec![idx];
        
        while let Some(current) = worklist.pop() {
            let children = match &self.objects[current].data {
                HeapData::Array(arr) => {
                    let mut c = Vec::new();
                    for val in arr {
                        if let Value::Array(i) = val { c.push(*i); }
                        if let Value::Kamus(i) = val { c.push(*i); }
                        if let Value::String(i) = val { c.push(*i); }
                        if let Value::Fungsi(i, _) = val { c.push(*i); }
                        if let Value::FungsiBawaan(i) = val { c.push(*i); }
                        if let Value::Modul(i) = val { c.push(*i); }
                    }
                    c
                },
                HeapData::Kamus(k) | HeapData::Modul(k) => {
                    let mut c = Vec::new();
                    for val in k.values() {
                        if let Value::Array(i) = val { c.push(*i); }
                        if let Value::Kamus(i) = val { c.push(*i); }
                        if let Value::String(i) = val { c.push(*i); }
                        if let Value::Fungsi(i, _) = val { c.push(*i); }
                        if let Value::FungsiBawaan(i) = val { c.push(*i); }
                        if let Value::Modul(i) = val { c.push(*i); }
                    }
                    c
                },
                HeapData::Fungsi(f) => {
                    let mut c = Vec::new();
                    for val in &f.chunk.constants {
                        if let Value::Array(i) = val { c.push(*i); }
                        if let Value::Kamus(i) = val { c.push(*i); }
                        if let Value::String(i) = val { c.push(*i); }
                        if let Value::Fungsi(i, _) = val { c.push(*i); }
                        if let Value::FungsiBawaan(i) = val { c.push(*i); }
                        if let Value::Modul(i) = val { c.push(*i); }
                    }
                    c
                },
                _ => Vec::new(),
            };
            
            for child in children {
                if !self.objects[child].is_marked {
                    self.objects[child].is_marked = true;
                    worklist.push(child);
                }
            }
        }
    }
    
    pub fn mark_sessions(&mut self) {
        let mut session_indices = Vec::new();
        for (_, (_, idx)) in &self.web_state.sessions {
            session_indices.push(*idx);
        }
        for idx in session_indices {
            self.mark(idx);
        }
    }

    pub fn sweep(&mut self) {
        for i in 0..self.objects.len() {
            if let HeapData::Free(_) = self.objects[i].data {
                continue;
            }
            if !self.objects[i].is_marked {
                self.free(i);
            } else {
                self.objects[i].is_marked = false; // unmark for next GC cycle
            }
        }
    }
}

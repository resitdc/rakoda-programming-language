use std::fmt;
use crate::heap::Heap;

pub trait VmContext {
    fn get_heap_mut(&mut self) -> &mut Heap;
    fn execute_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value, String>;
    fn spawn_task(&mut self, func: Value) -> Result<usize, String>;
    fn join_task(&mut self, task_id: usize) -> Result<Value, String>;
}

pub type NativeFnVM = fn(&mut dyn VmContext, Vec<Value>) -> Result<Value, String>;

#[derive(Clone)]
pub struct FungsiBawaanVM {
    pub nama: String,
    pub func: NativeFnVM,
}

impl PartialEq for FungsiBawaanVM {
    fn eq(&self, other: &Self) -> bool {
        self.nama == other.nama
    }
}

impl fmt::Debug for FungsiBawaanVM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fungsi bawaan {}>", self.nama)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FungsiVM {
    pub nama: String,
    pub parameter: Vec<String>,
    pub chunk: crate::compiler::Chunk,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Angka(f64),
    Boolean(bool),
    Kosong,
    String(usize),
    Array(usize),
    Kamus(usize),
    Fungsi(usize, usize),
    FungsiBawaan(usize),
    Modul(usize),
}

impl Value {
    pub fn to_string(&self, heap: &Heap) -> String {
        match self {
            Value::Angka(val) => val.to_string(),
            Value::String(idx) => heap.get_string(*idx).clone(),
            Value::Boolean(val) => (if *val { "benar" } else { "salah" }).to_string(),
            Value::Fungsi(idx, _) => format!("<fungsi {}>", heap.get_fungsi(*idx).nama),
            Value::FungsiBawaan(idx) => format!("<fungsi bawaan {}>", heap.get_fungsi_bawaan(*idx).nama),
            Value::Modul(_) => "<modul>".to_string(),
            Value::Array(idx) => {
                let items: Vec<String> = heap.get_array(*idx).iter().map(|v| v.to_string(heap)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Kamus(idx) => {
                let items: Vec<String> = heap.get_kamus(*idx).iter().map(|(k, v)| format!("{}: {}", k, v.to_string(heap))).collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Kosong => "kosong".to_string(),
        }
    }
}

pub fn deep_copy_value(val: &Value, source: &Heap, dest: &mut Heap) -> Value {
    match val {
        Value::Angka(n) => Value::Angka(*n),
        Value::Boolean(b) => Value::Boolean(*b),
        Value::Kosong => Value::Kosong,
        Value::String(idx) => {
            let s = source.get_string(*idx).clone();
            let new_idx = dest.alloc(crate::heap::HeapData::String(s));
            Value::String(new_idx)
        }
        Value::Array(idx) => {
            let arr = source.get_array(*idx).clone();
            let mut new_arr = Vec::new();
            for item in arr {
                new_arr.push(deep_copy_value(&item, source, dest));
            }
            let new_idx = dest.alloc(crate::heap::HeapData::Array(new_arr));
            Value::Array(new_idx)
        }
        Value::Kamus(idx) => {
            let dict = source.get_kamus(*idx).clone();
            let mut new_dict = std::collections::HashMap::new();
            for (k, v) in dict {
                new_dict.insert(k.clone(), deep_copy_value(&v, source, dest));
            }
            let new_idx = dest.alloc(crate::heap::HeapData::Kamus(new_dict));
            Value::Kamus(new_idx)
        }
        Value::Fungsi(idx, env) => {
            let f = source.get_fungsi(*idx).clone();
            let new_idx = dest.alloc(crate::heap::HeapData::Fungsi(f));
            Value::Fungsi(new_idx, *env)
        }
        Value::FungsiBawaan(idx) => {
            let f = source.get_fungsi_bawaan(*idx).clone();
            let new_idx = dest.alloc(crate::heap::HeapData::FungsiBawaan(f));
            Value::FungsiBawaan(new_idx)
        }
        Value::Modul(idx) => {
            Value::Modul(*idx)
        }
    }
}

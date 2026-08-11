use crate::heap::Heap;
use std::fmt;

pub trait VmContext {
    fn get_heap_mut(&mut self) -> &mut Heap;
    fn compile_source(&mut self, source: &str) -> Result<Value, String>;
    fn execute_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value, String>;
    fn spawn_task(&mut self, func: Value) -> Result<usize, String>;
    fn join_task(&mut self, task_id: usize) -> Result<Value, String>;
    fn as_any(&mut self) -> &mut dyn std::any::Any;
    fn current_lokasi(&self) -> Option<errors::Lokasi>;
    fn current_function_info(&self) -> (String, Option<String>);
}

pub type NativeFnVM =
    std::sync::Arc<dyn Fn(&mut dyn VmContext, Vec<Value>) -> Result<Value, String> + Send + Sync>;

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
    pub file: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Angka(f64),
    Kompleks(f64, f64),
    Boolean(bool),
    Kosong,
    String(usize),
    Array(usize),
    Kamus(usize),
    Fungsi(usize, usize),
    FungsiBawaan(usize),
    Modul(usize),
    DbPool(usize),
    QueryState(usize),
    Rentang(usize),
    Float64Array(usize),
    Int32Array(usize),
    RandomGenerator(usize),
}

impl Value {
    pub fn to_string(&self, heap: &Heap) -> String {
        match self {
            Value::Angka(val) => val.to_string(),
            Value::Kompleks(real, imag) => {
                if *real == 0.0 {
                    format!("{}j", imag)
                } else if *imag >= 0.0 {
                    format!("{} + {}j", real, imag)
                } else {
                    format!("{} - {}j", real, -imag)
                }
            },
            Value::String(idx) => heap.get_string(*idx).clone(),
            Value::Boolean(val) => (if *val { "benar" } else { "salah" }).to_string(),
            Value::Fungsi(idx, _) => format!("<fungsi {}>", heap.get_fungsi(*idx).nama),
            Value::FungsiBawaan(idx) => {
                format!("<fungsi bawaan {}>", heap.get_fungsi_bawaan(*idx).nama)
            }
            Value::Modul(_) => "<modul>".to_string(),
            Value::Array(idx) => {
                let items: Vec<String> = heap
                    .get_array(*idx)
                    .iter()
                    .map(|v| v.to_string(heap))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Value::Kamus(idx) => {
                let items: Vec<String> = heap
                    .get_kamus(*idx)
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string(heap)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Kosong => "kosong".to_string(),
            Value::DbPool(_) => "<koneksi database>".to_string(),
            Value::QueryState(_) => "<query builder>".to_string(),
            Value::Rentang(idx) => {
                let (mulai, sampai) = heap.get_rentang(*idx);
                let ms = mulai
                    .as_ref()
                    .map(|v| v.to_string(heap))
                    .unwrap_or_else(|| "".to_string());
                let ss = sampai
                    .as_ref()
                    .map(|v| v.to_string(heap))
                    .unwrap_or_else(|| "".to_string());
                format!("{}:{}", ms, ss)
            }
            Value::Float64Array(idx) => {
                let t = heap.get_f64_tensor(*idx);
                let _lock = t.data.read().unwrap();
                format!("<Float64Array bentuk={:?}>", t.shape)
            }
            Value::Int32Array(idx) => {
                let t = heap.get_i32_tensor(*idx);
                format!("<Int32Array bentuk={:?}>", t.shape)
            }
            Value::RandomGenerator(_) => "<random generator>".to_string(),
        }
    }
}

pub fn deep_copy_value(val: &Value, source: &Heap, dest: &mut Heap) -> Value {
    match val {
        Value::Angka(n) => Value::Angka(*n),
        Value::Kompleks(real, imag) => Value::Kompleks(*real, *imag),
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
        Value::Modul(idx) => Value::Modul(*idx),
        Value::DbPool(idx) => Value::DbPool(*idx),
        Value::QueryState(idx) => Value::QueryState(*idx),
        Value::Rentang(idx) => {
            let (m, s) = source.get_rentang(*idx);
            let nm = m.as_ref().map(|v| deep_copy_value(v, source, dest));
            let ns = s.as_ref().map(|v| deep_copy_value(v, source, dest));
            let new_idx = dest.alloc(crate::heap::HeapData::Rentang(nm, ns));
            Value::Rentang(new_idx)
        }
        Value::Float64Array(idx) => {
            let t = source.get_f64_tensor(*idx).clone();
            let new_idx = dest.alloc(crate::heap::HeapData::Float64Array(t));
            Value::Float64Array(new_idx)
        }
        Value::Int32Array(idx) => {
            let t = source.get_i32_tensor(*idx).clone();
            let new_idx = dest.alloc(crate::heap::HeapData::Int32Array(t));
            Value::Int32Array(new_idx)
        }
        Value::RandomGenerator(idx) => {
            let rng = source.get_random_generator(*idx).borrow().clone();
            let new_idx = dest.alloc(crate::heap::HeapData::RandomGenerator(std::cell::RefCell::new(rng)));
            Value::RandomGenerator(new_idx)
        }
    }
}

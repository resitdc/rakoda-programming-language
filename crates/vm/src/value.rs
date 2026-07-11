use std::rc::Rc;
use std::fmt;

use std::collections::HashMap;
use std::cell::RefCell;

pub type NativeFnVM = fn(Vec<Value>) -> Result<Value, String>;

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

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Angka(f64),
    String(Rc<String>),
    Boolean(bool),
    Fungsi(Rc<FungsiVM>),
    FungsiBawaan(Rc<FungsiBawaanVM>),
    Array(Rc<RefCell<Vec<Value>>>),
    Kamus(Rc<RefCell<HashMap<String, Value>>>),
    Kosong,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Angka(val) => write!(f, "{}", val),
            Value::String(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "{}", if *val { "benar" } else { "salah" }),
            Value::Fungsi(fungsi) => write!(f, "<fungsi {}>", fungsi.nama),
            Value::FungsiBawaan(fungsi) => write!(f, "<fungsi bawaan {}>", fungsi.nama),
            Value::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Kamus(kamus) => {
                let items: Vec<String> = kamus.borrow().iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Kosong => write!(f, "kosong"),
        }
    }
}

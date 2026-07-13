//! Thin adapter: wraps stdlib crate's json module for interpreter use.
//! Also provides Objek↔serde_json::Value converters needed by HTTP module.

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use crate::stdlib::adapter::bungkus_fungsi;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use stdlib;

pub(crate) fn to_json(obj: &Objek) -> Value {
    match obj {
        Objek::String(s) => Value::String(s.clone()),
        Objek::Angka(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Value::Number(num)
            } else {
                Value::Null
            }
        }
        Objek::Boolean(b) => Value::Bool(*b),
        Objek::Kosong => Value::Null,
        Objek::Array(arr) => {
            let vec: Vec<Value> = arr.borrow().iter().map(to_json).collect();
            Value::Array(vec)
        }
        Objek::Kamus(map) => {
            let mut obj_map = serde_json::Map::new();
            for (k, v) in map.borrow().iter() {
                obj_map.insert(k.clone(), to_json(v));
            }
            Value::Object(obj_map)
        }
        _ => Value::Null, // ignore functions/modules
    }
}

pub(crate) fn from_json(val: &Value) -> Objek {
    match val {
        Value::Null => Objek::Kosong,
        Value::Bool(b) => Objek::Boolean(*b),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Objek::Angka(f)
            } else {
                Objek::Kosong
            }
        }
        Value::String(s) => Objek::String(s.clone()),
        Value::Array(arr) => {
            let vec: Vec<Objek> = arr.iter().map(from_json).collect();
            Objek::Array(Rc::new(RefCell::new(vec)))
        }
        Value::Object(map) => {
            let mut hash = HashMap::new();
            for (k, v) in map {
                hash.insert(k.clone(), from_json(v));
            }
            Objek::Kamus(Rc::new(RefCell::new(hash)))
        }
    }
}

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    for (nama, f) in stdlib::json::fungsi_json() {
        module_env
            .borrow_mut()
            .set(nama.to_string(), Objek::MetodeBawaan(bungkus_fungsi(f)));
    }

    env.borrow_mut()
        .set("json".to_string(), Objek::Modul(module_env));
}

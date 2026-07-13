use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

    // json.buat(object) -> string
    module_env.borrow_mut().set(
        "buat".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(arg) = args.first() {
                let json_val = to_json(arg);
                if let Ok(s) = serde_json::to_string(&json_val) {
                    return Objek::String(s);
                }
            }
            Objek::Kosong
        }),
    );

    // json.parse(string) -> object
    module_env.borrow_mut().set(
        "parse".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 1
                && let Objek::String(s) = &args[0]
                    && let Ok(val) = serde_json::from_str::<Value>(s) {
                        return from_json(&val);
                    }
            Objek::Kosong
        }),
    );

    env.borrow_mut()
        .set("json".to_string(), Objek::Modul(module_env));
}

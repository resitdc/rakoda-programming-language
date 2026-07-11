use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use serde_json::Value as JsonValue;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    // json.parse(teks)
    let parse_func = FungsiBawaanVM {
        nama: "parse".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("Fungsi 'parse' membutuhkan 1 argumen: teks".to_string());
            }
            if let Value::String(s) = &args[0] {
                match serde_json::from_str::<JsonValue>(s) {
                    Ok(json_val) => Ok(json_to_value(&json_val)),
                    Err(e) => Err(format!("Gagal mem-parsing JSON: {}", e)),
                }
            } else {
                Err("Argumen harus berupa teks".to_string())
            }
        },
    };
    module_dict.insert("parse".to_string(), Value::FungsiBawaan(Rc::new(parse_func)));

    // json.stringify(data)
    let stringify_func = FungsiBawaanVM {
        nama: "stringify".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("Fungsi 'stringify' membutuhkan 1 argumen: data".to_string());
            }
            let json_val = value_to_json(&args[0]);
            match serde_json::to_string(&json_val) {
                Ok(s) => Ok(Value::String(Rc::new(s))),
                Err(e) => Err(format!("Gagal men-stringify JSON: {}", e)),
            }
        },
    };
    module_dict.insert("stringify".to_string(), Value::FungsiBawaan(Rc::new(stringify_func)));

    vm.set_global("json".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Kosong,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Angka(f)
            } else {
                Value::Kosong
            }
        },
        JsonValue::String(s) => Value::String(Rc::new(s.clone())),
        JsonValue::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                vec.push(json_to_value(item));
            }
            Value::Array(Rc::new(RefCell::new(vec)))
        },
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Kamus(Rc::new(RefCell::new(map)))
        },
    }
}

fn value_to_json(val: &Value) -> JsonValue {
    match val {
        Value::Kosong => JsonValue::Null,
        Value::Boolean(b) => JsonValue::Bool(*b),
        Value::Angka(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                JsonValue::Number(num)
            } else {
                JsonValue::Null
            }
        },
        Value::String(s) => JsonValue::String(s.to_string()),
        Value::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr.borrow().iter() {
                vec.push(value_to_json(item));
            }
            JsonValue::Array(vec)
        },
        Value::Kamus(kamus) => {
            let mut map = serde_json::Map::new();
            for (k, v) in kamus.borrow().iter() {
                map.insert(k.clone(), value_to_json(v));
            }
            JsonValue::Object(map)
        },
        _ => JsonValue::Null,
    }
}

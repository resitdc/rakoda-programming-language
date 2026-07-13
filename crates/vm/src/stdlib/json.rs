use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub fn convert_to_value(ctx: &mut dyn VmContext, json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Kosong,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(num) => {
            if let Some(n) = num.as_f64() {
                Value::Angka(n)
            } else {
                Value::Kosong
            }
        }
        JsonValue::String(s) => {
            let idx = ctx.get_heap_mut().alloc(HeapData::String(s.clone()));
            Value::String(idx)
        }
        JsonValue::Array(arr) => {
            let elements = arr.iter().map(|v| convert_to_value(ctx, v)).collect();
            let idx = ctx.get_heap_mut().alloc(HeapData::Array(elements));
            Value::Array(idx)
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), convert_to_value(ctx, v));
            }
            let idx = ctx.get_heap_mut().alloc(HeapData::Kamus(map));
            Value::Kamus(idx)
        }
    }
}

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let parse_func = FungsiBawaanVM {
        nama: "parse".to_string(),
        func: |ctx, args| {
            if args.is_empty() {
                return Err("Fungsi 'parse' membutuhkan 1 argumen: json string".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let s = ctx.get_heap_mut().get_string(*idx).clone();
                match serde_json::from_str::<JsonValue>(&s) {
                    Ok(json_val) => Ok(convert_to_value(ctx, &json_val)),
                    Err(e) => Err(format!("Gagal mem-parsing JSON: {}", e)),
                }
            } else {
                Err("Argumen harus berupa teks".to_string())
            }
        },
    };
    let parse_idx = vm.heap.alloc(HeapData::FungsiBawaan(parse_func));
    module_dict.insert("parse".to_string(), Value::FungsiBawaan(parse_idx));

    let stringify_func = FungsiBawaanVM {
        nama: "stringify".to_string(),
        func: |ctx, args| {
            if args.is_empty() {
                return Err("Fungsi 'stringify' membutuhkan 1 argumen: data".to_string());
            }

            fn convert_from_value(ctx: &mut dyn VmContext, val: &Value) -> JsonValue {
                match val {
                    Value::Kosong => JsonValue::Null,
                    Value::Boolean(b) => JsonValue::Bool(*b),
                    Value::Angka(n) => {
                        if let Some(num) = serde_json::Number::from_f64(*n) {
                            JsonValue::Number(num)
                        } else {
                            JsonValue::Null
                        }
                    }
                    Value::String(idx) => {
                        JsonValue::String(ctx.get_heap_mut().get_string(*idx).clone())
                    }
                    Value::Array(idx) => {
                        let array_clone = ctx.get_heap_mut().get_array(*idx).clone();
                        let elements = array_clone
                            .iter()
                            .map(|v| convert_from_value(ctx, v))
                            .collect();
                        JsonValue::Array(elements)
                    }
                    Value::Kamus(idx) => {
                        let mut map = serde_json::Map::new();
                        let kamus_clone = ctx.get_heap_mut().get_kamus(*idx).clone();
                        for (k, v) in kamus_clone {
                            map.insert(k, convert_from_value(ctx, &v));
                        }
                        JsonValue::Object(map)
                    }
                    Value::Fungsi(..) | Value::FungsiBawaan(_) | Value::Modul(_) => JsonValue::Null,
                }
            }

            let json_val = convert_from_value(ctx, &args[0]);
            let s = json_val.to_string();
            let s_idx = ctx.get_heap_mut().alloc(HeapData::String(s));
            Ok(Value::String(s_idx))
        },
    };
    let stringify_idx = vm
        .heap
        .alloc(HeapData::FungsiBawaan(stringify_func.clone()));
    module_dict.insert("stringify".to_string(), Value::FungsiBawaan(stringify_idx));

    let buat_idx = vm.heap.alloc(HeapData::FungsiBawaan(stringify_func));
    module_dict.insert("buat".to_string(), Value::FungsiBawaan(buat_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("json".to_string(), Value::Kamus(dict_idx));
}

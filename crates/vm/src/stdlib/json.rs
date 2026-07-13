//! Thin adapter: wraps stdlib crate's json module for VM use.
//! Also provides serde_json::Value↔Value converter needed by web module.
use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use serde_json;
use std::collections::HashMap;

/// Convert serde_json::Value → RPL Value (used by web request parsing).
pub fn convert_to_value(vm: &mut VM, json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Kosong,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Angka(f)
            } else {
                Value::Kosong
            }
        }
        serde_json::Value::String(s) => {
            let idx = vm.heap.alloc(HeapData::String(s.clone()));
            Value::String(idx)
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(|v| convert_to_value(vm, v)).collect();
            let idx = vm.heap.alloc(HeapData::Array(items));
            Value::Array(idx)
        }
        serde_json::Value::Object(map) => {
            let mut hash = HashMap::new();
            for (k, v) in map {
                hash.insert(k.clone(), convert_to_value(vm, v));
            }
            let idx = vm.heap.alloc(HeapData::Kamus(hash));
            Value::Kamus(idx)
        }
    }
}

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    for (nama, func) in stdlib::json::fungsi_json() {
        let fungsi = FungsiBawaanVM {
            nama: nama.to_string(),
            func: unsafe {
                std::mem::transmute(
                    move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                        let heap = ctx.get_heap_mut();
                        let nilai_args: Vec<stdlib::jenis::NilaiRpl> = args
                            .iter()
                            .map(|v| adapter::value_ke_nilai(v, heap))
                            .collect();
                        match func(&nilai_args) {
                            Ok(result) => {
                                let heap2 = ctx.get_heap_mut();
                                Ok(adapter::nilai_ke_value(&result, heap2))
                            }
                            Err(e) => Err(e),
                        }
                    },
                )
            },
        };
        let idx = vm.heap.alloc(HeapData::FungsiBawaan(fungsi));
        module_dict.insert(nama.to_string(), Value::FungsiBawaan(idx));
    }

    let module_idx = vm.heap.alloc(HeapData::Modul(module_dict));
    vm.set_global("json".to_string(), Value::Modul(module_idx));
}

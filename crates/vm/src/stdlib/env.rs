use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;
use std::env;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let _ = dotenvy::dotenv();

    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.is_empty() {
                return Err("Fungsi 'get' membutuhkan 1 argumen: kunci (key)".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let key = ctx.get_heap_mut().get_string(*idx).clone();
                match env::var(&key) {
                    Ok(val) => {
                        let new_idx = ctx.get_heap_mut().alloc(HeapData::String(val));
                        Ok(Value::String(new_idx))
                    }
                    Err(_) => Ok(Value::Kosong),
                }
            } else {
                Err("Kunci (key) harus berupa teks".to_string())
            }
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    module_dict.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let set_func = FungsiBawaanVM {
        nama: "set".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err(
                    "Fungsi 'set' membutuhkan 2 argumen: nama_variabel dan nilai".to_string(),
                );
            }
            if let (Value::String(k_idx), Value::String(v_idx)) = (&args[0], &args[1]) {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();
                let val = ctx.get_heap_mut().get_string(*v_idx).clone();
                unsafe {
                    std::env::set_var(key, val);
                }
                Ok(Value::Kosong)
            } else {
                Err("Nama variabel dan nilai harus berupa teks".to_string())
            }
        },
    };
    let set_idx = vm.heap.alloc(HeapData::FungsiBawaan(set_func));
    module_dict.insert("set".to_string(), Value::FungsiBawaan(set_idx));

    let load_func = FungsiBawaanVM {
        nama: "load".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'load' membutuhkan 1 argumen: path".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let path = ctx.get_heap_mut().get_string(*idx).clone();
                match dotenvy::from_path(&path) {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("Gagal memuat file .env '{}': {}", path, e)),
                }
            } else {
                Err("Path harus berupa teks".to_string())
            }
        },
    };
    let load_idx = vm.heap.alloc(HeapData::FungsiBawaan(load_func));
    module_dict.insert("load".to_string(), Value::FungsiBawaan(load_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("env".to_string(), Value::Kamus(dict_idx));
}

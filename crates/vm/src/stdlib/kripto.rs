use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use md5::{Digest as Md5Digest, Md5};
use sha2::Sha256;
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let md5_func = FungsiBawaanVM {
        nama: "md5".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("kripto.md5 membutuhkan 1 argumen: teks".to_string());
            }
            let teks = match &args[0] {
                Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("kripto.md5 hanya menerima teks".to_string()),
            };

            let mut hasher = Md5::new();
            hasher.update(teks.as_bytes());
            let result = hasher.finalize();
            let hex_string = format!("{:x}", result);

            let str_idx = vm
                .get_heap_mut()
                .alloc(crate::heap::HeapData::String(hex_string));
            Ok(Value::String(str_idx))
        },
    };

    let sha256_func = FungsiBawaanVM {
        nama: "sha256".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("kripto.sha256 membutuhkan 1 argumen: teks".to_string());
            }
            let teks = match &args[0] {
                Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("kripto.sha256 hanya menerima teks".to_string()),
            };

            let mut hasher = Sha256::new();
            hasher.update(teks.as_bytes());
            let result = hasher.finalize();
            let hex_string = format!("{:x}", result);

            let str_idx = vm
                .get_heap_mut()
                .alloc(crate::heap::HeapData::String(hex_string));
            Ok(Value::String(str_idx))
        },
    };

    let md5_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(md5_func));
    module_dict.insert("md5".to_string(), Value::FungsiBawaan(md5_idx));

    let sha256_idx = vm
        .heap
        .alloc(crate::heap::HeapData::FungsiBawaan(sha256_func));
    module_dict.insert("sha256".to_string(), Value::FungsiBawaan(sha256_idx));

    let module_idx = vm.heap.alloc(crate::heap::HeapData::Modul(module_dict));
    vm.set_global("kripto".to_string(), Value::Modul(module_idx));
}

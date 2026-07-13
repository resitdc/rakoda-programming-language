use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.is_empty() {
                return Err("Fungsi 'get' membutuhkan minimal 1 argumen: url".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let url = ctx.get_heap_mut().get_string(*idx).clone();
                match ureq::get(&url).call() {
                    Ok(mut response) => {
                        let mut resp_dict = HashMap::new();
                        resp_dict.insert(
                            "status".to_string(),
                            Value::Angka(response.status().as_u16() as f64),
                        );

                        let body = response.body_mut().read_to_string().unwrap_or_default();
                        let body_idx = ctx.get_heap_mut().alloc(HeapData::String(body));
                        resp_dict.insert("body".to_string(), Value::String(body_idx));

                        let dict_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(resp_dict));
                        Ok(Value::Kamus(dict_idx))
                    }
                    Err(e) => Err(format!("Permintaan HTTP gagal: {}", e)),
                }
            } else {
                Err("URL harus berupa teks".to_string())
            }
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    module_dict.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("http".to_string(), Value::Kamus(dict_idx));
}

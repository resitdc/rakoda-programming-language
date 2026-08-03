//! VM module registration for file.
//! Delegates all logic to crates/stdlib/file.rs via adapter.
use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    for (nama, func_ptr) in stdlib::file::fungsi_file() {
        let nama_copy = nama.to_string();
        let fungsi = FungsiBawaanVM {
            nama: nama.to_string(),
            func: std::sync::Arc::new(
                move |ctx: &mut dyn VmContext, mut args: Vec<Value>| -> Result<Value, String> {
                    // Resolve paths in arguments
                    let path_indices = match nama_copy.as_str() {
                        "baca" | "tulis" | "ada" | "hapus" | "buat_folder" | "daftar" => vec![0],
                        "pindah" => vec![0, 1],
                        _ => vec![],
                    };

                    for i in path_indices {
                        if i < args.len() {
                            let path_str = if let Value::String(idx) = args[i] {
                                Some(ctx.get_heap_mut().get_string(idx).clone())
                            } else {
                                None
                            };
                            
                            if let Some(p) = path_str {
                                let new_path = adapter::resolve_path(ctx, &p);
                                let new_idx = ctx.get_heap_mut().alloc(HeapData::String(new_path));
                                args[i] = Value::String(new_idx);
                            }
                        }
                    }

                    let heap = ctx.get_heap_mut();
                    let nilai_args: Vec<stdlib::jenis::NilaiRpl> = args
                        .iter()
                        .map(|v| adapter::value_ke_nilai(v, heap))
                        .collect();
                    match func_ptr(&nilai_args) {
                        Ok(result) => {
                            let heap2 = ctx.get_heap_mut();
                            Ok(adapter::nilai_ke_value(&result, heap2))
                        }
                        Err(e) => Err(e),
                    }
                },
            ),
        };
        let idx = vm.heap.alloc(HeapData::FungsiBawaan(fungsi));
        module_dict.insert(nama.to_string(), Value::FungsiBawaan(idx));
    }

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("file".to_string(), Value::Kamus(dict_idx));
}

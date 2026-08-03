use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    for (nama, func_ref) in &stdlib::dokumen::fungsi_dokumen() {
        let func_ptr = *func_ref;
        let nama_copy = nama.to_string();
        let fungsi = FungsiBawaanVM {
            nama: nama.to_string(),
            func: std::sync::Arc::new(
                move |ctx: &mut dyn VmContext, mut args: Vec<Value>| -> Result<Value, String> {
                    if nama_copy == "baca" && !args.is_empty() {
                        let path_str = if let Value::String(idx) = args[0] {
                            Some(ctx.get_heap_mut().get_string(idx).clone())
                        } else {
                            None
                        };
                        
                        if let Some(p) = path_str {
                            let new_path = adapter::resolve_path(ctx, &p);
                            let new_idx = ctx.get_heap_mut().alloc(HeapData::String(new_path));
                            args[0] = Value::String(new_idx);
                        }
                    }

                    let heap = ctx.get_heap_mut();
                    let nilai_args: Vec<stdlib::NilaiRpl> = args
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
    vm.set_global("dokumen".to_string(), Value::Kamus(dict_idx));
}

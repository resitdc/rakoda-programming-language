//! Thin adapter: wraps shared stdlib's list module for VM use.

use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    for (nama, func) in &stdlib::list::fungsi_list() {
        let fungsi = FungsiBawaanVM {
            nama: nama.to_string(),
            func: unsafe {
                std::mem::transmute(
                    move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                        let heap = ctx.get_heap_mut();
                        let nilai_args: Vec<stdlib::NilaiRpl> = args
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

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("list".to_string(), Value::Kamus(dict_idx));
}

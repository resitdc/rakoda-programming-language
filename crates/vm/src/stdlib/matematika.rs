//! VM module registration for matematika.
//! Generates thin fn-pointer wrappers via macro that delegate to crates/stdlib.

use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let fungsi_list = stdlib::matematika::fungsi_matematika();
    for (nama, func_ref) in &fungsi_list {
        // Use unsafe transmute to convert closure to fn pointer
        // This is safe because the closure captures nothing (non-capturing).
        let func_ptr = *func_ref;
        let fungsi = FungsiBawaanVM {
            nama: nama.to_string(),
            func: std::sync::Arc::new(
                move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
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
    vm.set_global("matematika".to_string(), Value::Kamus(dict_idx));
    
    // Also register Matematika for uppercase (often users capitalize Math modules)
    vm.set_global("Matematika".to_string(), Value::Kamus(dict_idx));

    // Add Mathematical Constants to the dictionary directly
    if let HeapData::Kamus(ref mut dict) = vm.heap.objects[dict_idx].data {
        dict.insert("PI".to_string(), Value::Angka(std::f64::consts::PI));
        dict.insert("E".to_string(), Value::Angka(std::f64::consts::E));
        dict.insert("Infinity".to_string(), Value::Angka(std::f64::INFINITY));
        dict.insert("NaN".to_string(), Value::Angka(std::f64::NAN));
    }
}

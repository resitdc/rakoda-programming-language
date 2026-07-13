//! Thin adapter: wraps shared stdlib's string module for VM use.
//! "dari" is VM-specific (converts any Value to string) and kept as helper.

use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    // Pure delegation for panjang, besar, kecil, potong, ganti via unsafe transmute
    // (pattern established in matematika.rs / core.rs — NativeFnVM = fn pointer type)
    for (nama, func) in &stdlib::string::fungsi_string() {
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

    // "dari" — VM-specific: converts any Value to String (no pure shared equivalent)
    fn dari_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("string.dari membutuhkan 1 argumen".to_string());
        }
        let s = args[0].to_string(ctx.get_heap_mut());
        let idx = ctx.get_heap_mut().alloc(HeapData::String(s));
        Ok(Value::String(idx))
    }
    let dari = FungsiBawaanVM {
        nama: "dari".to_string(),
        func: dari_wrapper,
    };
    let dari_idx = vm.heap.alloc(HeapData::FungsiBawaan(dari));
    module_dict.insert("dari".to_string(), Value::FungsiBawaan(dari_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("string".to_string(), Value::Kamus(dict_idx));
}

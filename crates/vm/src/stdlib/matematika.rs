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

    // Register Matematika.turunan (AutoGrad)
    let turunan_func = FungsiBawaanVM {
        nama: "turunan".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err("Matematika.turunan membutuhkan 2 argumen: (fungsi, nilai_x)".to_string());
                }
                let func = args[0];
                let x_val = args[1];
                let x = match x_val {
                    Value::Angka(n) => n,
                    _ => return Err("Nilai x harus berupa angka".to_string()),
                };
                
                let dual_x = Value::AngkaDual(x, 1.0);
                let result = ctx.execute_function(func, vec![dual_x])?;
                
                match result {
                    Value::AngkaDual(_, grad) => Ok(Value::Angka(grad)),
                    Value::Angka(_) => Ok(Value::Angka(0.0)), // Constant function returns 0.0 derivative
                    _ => Err("Fungsi tidak mengembalikan nilai numerik yang dapat diturunkan".to_string()),
                }
            },
        ),
    };
    let turunan_idx = vm.heap.alloc(HeapData::FungsiBawaan(turunan_func));

    // Add Mathematical Constants and Special VM Functions to the dictionary directly
    if let HeapData::Kamus(ref mut dict) = vm.heap.objects[dict_idx].data {
        dict.insert("PI".to_string(), Value::Angka(std::f64::consts::PI));
        dict.insert("E".to_string(), Value::Angka(std::f64::consts::E));
        dict.insert("Infinity".to_string(), Value::Angka(std::f64::INFINITY));
        dict.insert("NaN".to_string(), Value::Angka(std::f64::NAN));
        dict.insert("turunan".to_string(), Value::FungsiBawaan(turunan_idx));
    }
}

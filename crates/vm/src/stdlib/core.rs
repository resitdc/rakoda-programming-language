use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};

pub fn register(vm: &mut VM) {
    let angka_func = FungsiBawaanVM {
        nama: "angka".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'angka' membutuhkan 1 argumen".to_string());
            }
            match &args[0] {
                Value::Angka(n) => Ok(Value::Angka(*n)),
                Value::String(idx) => {
                    let s = ctx.get_heap_mut().get_string(*idx).clone();
                    if let Ok(n) = s.parse::<f64>() {
                        Ok(Value::Angka(n))
                    } else {
                        Err(format!("Tidak dapat mengubah '{}' menjadi angka", s))
                    }
                }
                _ => Err("Argumen tidak didukung".to_string()),
            }
        },
    };

    let angka_idx = vm.heap.alloc(HeapData::FungsiBawaan(angka_func));
    vm.set_global("angka".to_string(), Value::FungsiBawaan(angka_idx));

    let teks_func = FungsiBawaanVM {
        nama: "teks".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'teks' membutuhkan 1 argumen".to_string());
            }
            let s = args[0].to_string(ctx.get_heap_mut());
            let idx = ctx.get_heap_mut().alloc(HeapData::String(s));
            Ok(Value::String(idx))
        },
    };

    let teks_idx = vm.heap.alloc(HeapData::FungsiBawaan(teks_func));
    vm.set_global("teks".to_string(), Value::FungsiBawaan(teks_idx));
}

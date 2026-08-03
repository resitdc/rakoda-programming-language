//! Thin adapter: wraps shared stdlib's waktu module for VM use.
//! Pure functions delegated to crates/stdlib. `tunggu` and `string` are VM-specific.

use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    // Pure delegation for sekarang, tahun, bulan, tanggal, jam, menit, detik, format
    for (nama, func_ref) in &stdlib::waktu::fungsi_waktu() {
        let func_ptr = *func_ref;
        let fungsi = FungsiBawaanVM {
            nama: nama.to_string(),
            func: std::sync::Arc::new(
                move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
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

    // "tunggu" — I/O specific (sleep), not in shared stdlib
    fn tunggu_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() < 1 || args.len() > 2 {
            return Err(
                "waktu.tunggu membutuhkan 1 atau 2 argumen: milidetik, dan fungsi (opsional)"
                    .to_string(),
            );
        }

        let ms = match &args[0] {
            Value::Angka(ms) => *ms as u64,
            v => {
                let ms_str = v.to_string(ctx.get_heap_mut());
                ms_str.parse::<f64>().map(|v| v as u64).map_err(|e| {
                    format!(
                        "waktu.tunggu: gagal mengubah argumen pertama ke angka: {}",
                        e
                    )
                })?
            }
        };

        std::thread::sleep(std::time::Duration::from_millis(ms));

        if args.len() == 2 {
            let func = args[1].clone();
            ctx.execute_function(func, vec![])?;
        }

        Ok(Value::Kosong)
    }
    let tunggu = FungsiBawaanVM {
        nama: "tunggu".to_string(),
        func: std::sync::Arc::new(tunggu_wrapper),
    };
    let tunggu_idx = vm.heap.alloc(HeapData::FungsiBawaan(tunggu));
    module_dict.insert("tunggu".to_string(), Value::FungsiBawaan(tunggu_idx));

    // "string" — VM-specific: returns formatted date string in heap
    fn string_wrapper(ctx: &mut dyn VmContext, _args: Vec<Value>) -> Result<Value, String> {
        let s = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let idx = ctx.get_heap_mut().alloc(HeapData::String(s));
        Ok(Value::String(idx))
    }
    let string = FungsiBawaanVM {
        nama: "string".to_string(),
        func: std::sync::Arc::new(string_wrapper),
    };
    let string_idx = vm.heap.alloc(HeapData::FungsiBawaan(string));
    module_dict.insert("string".to_string(), Value::FungsiBawaan(string_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("waktu".to_string(), Value::Kamus(dict_idx));
}

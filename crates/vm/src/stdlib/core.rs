//! Thin adapter: wraps shared stdlib's core module for VM use.
//! `tampilkan` and `baca` need I/O wrappers; `angka`, `teks`, `boolean` are pure delegation.

use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;
use std::io::{self, Write};

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    // Pure delegation for angka, teks, boolean via unsafe transmute
    // (pattern established in matematika.rs — needed because NativeFnVM = fn pointer,
    //  and closures that capture can't coerce to fn pointer types)
    for (nama, func) in &stdlib::core::fungsi_core() {
        // Skip tampilkan and baca — they need I/O wrappers
        if *nama == "tampilkan" || *nama == "baca" {
            continue;
        }
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

    // "tampilkan" with I/O wrapper (no capture — pure fn pointer)
    fn tampilkan_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        // Re-implement formatting logic from stdlib/src/core.rs:tampilkan_impl
        let heap = ctx.get_heap_mut();
        let output = args
            .iter()
            .map(|v| v.to_string(heap))
            .collect::<Vec<_>>()
            .join("");
        print!("{}", output);
        io::stdout().flush().map_err(|e| e.to_string())?;
        Ok(Value::Kosong)
    }
    let tampilkan = FungsiBawaanVM {
        nama: "tampilkan".to_string(),
        func: tampilkan_wrapper,
    };
    let tampilkan_idx = vm.heap.alloc(HeapData::FungsiBawaan(tampilkan));
    module_dict.insert("tampilkan".to_string(), Value::FungsiBawaan(tampilkan_idx));

    // "baca" with I/O wrapper (no capture — pure fn pointer)
    fn baca_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if let Some(prompt) = args.first() {
            let p = prompt.to_string(ctx.get_heap_mut());
            if !p.is_empty() {
                print!("{}", p);
                io::stdout().flush().map_err(|e| e.to_string())?;
            }
        }
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| e.to_string())?;
        let idx = ctx
            .get_heap_mut()
            .alloc(HeapData::String(input.trim_end().to_string()));
        Ok(Value::String(idx))
    }
    let baca = FungsiBawaanVM {
        nama: "baca".to_string(),
        func: baca_wrapper,
    };
    let baca_idx = vm.heap.alloc(HeapData::FungsiBawaan(baca));
    module_dict.insert("baca".to_string(), Value::FungsiBawaan(baca_idx));

    let module_idx = vm.heap.alloc(HeapData::Modul(module_dict));
    vm.set_global("core".to_string(), Value::Modul(module_idx));
}

use std::collections::HashMap;
use crate::value::{Value, FungsiBawaanVM};
use crate::heap::HeapData;

pub fn register(vm: &mut crate::machine::VM) {
    let mut methods = HashMap::new();

    methods.insert(
        "jalankan".to_string(),
        Value::FungsiBawaan(vm.heap.alloc(HeapData::FungsiBawaan(FungsiBawaanVM {
            nama: "tugas.jalankan".to_string(),
            func: |ctx, args| {
                if args.len() != 1 {
                    return Err("tugas.jalankan membutuhkan 1 argumen (fungsi)".to_string());
                }

                if let Value::Fungsi(_, _) = args[0] {
                    let task_id = ctx.spawn_task(args[0])?;
                    Ok(Value::Angka(task_id as f64))
                } else {
                    Err("Argumen pertama tugas.jalankan harus berupa fungsi".to_string())
                }
            },
        }))),
    );

    methods.insert(
        "tunggu".to_string(),
        Value::FungsiBawaan(vm.heap.alloc(HeapData::FungsiBawaan(FungsiBawaanVM {
            nama: "tugas.tunggu".to_string(),
            func: |ctx, args| {
                if args.len() != 1 {
                    return Err("tugas.tunggu membutuhkan 1 argumen (tiket tugas)".to_string());
                }

                if let Value::Angka(task_id_f) = args[0] {
                    let task_id = task_id_f as usize;
                    ctx.join_task(task_id)
                } else {
                    Err("Argumen tugas.tunggu harus berupa ID tugas (Angka)".to_string())
                }
            },
        }))),
    );

    let lib_idx = vm.heap.alloc(HeapData::Kamus(methods));
    vm.set_global("tugas".to_string(), Value::Kamus(lib_idx));
}

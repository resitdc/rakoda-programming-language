use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;
use std::fs;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let baca_func = FungsiBawaanVM {
        nama: "baca".to_string(),
        func: |ctx, args| {
            if args.is_empty() {
                return Err("Fungsi 'baca' membutuhkan 1 argumen: path".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let path = ctx.get_heap_mut().get_string(*idx).clone();
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        let new_idx = ctx.get_heap_mut().alloc(HeapData::String(content));
                        Ok(Value::String(new_idx))
                    }
                    Err(e) => Err(format!("Gagal membaca file '{}': {}", path, e)),
                }
            } else {
                Err("Path harus berupa teks".to_string())
            }
        },
    };
    let baca_idx = vm.heap.alloc(HeapData::FungsiBawaan(baca_func));
    module_dict.insert("baca".to_string(), Value::FungsiBawaan(baca_idx));

    let tulis_func = FungsiBawaanVM {
        nama: "tulis".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'tulis' membutuhkan 2 argumen: path dan isi".to_string());
            }
            if let (Value::String(path_idx), Value::String(content_idx)) = (&args[0], &args[1]) {
                let path = ctx.get_heap_mut().get_string(*path_idx).clone();
                let content = ctx.get_heap_mut().get_string(*content_idx).clone();
                match fs::write(&path, content.as_bytes()) {
                    Ok(_) => Ok(Value::Kosong),
                    Err(e) => Err(format!("Gagal menulis ke file '{}': {}", path, e)),
                }
            } else {
                Err("Path dan isi harus berupa teks".to_string())
            }
        },
    };
    let tulis_idx = vm.heap.alloc(HeapData::FungsiBawaan(tulis_func));
    module_dict.insert("tulis".to_string(), Value::FungsiBawaan(tulis_idx));

    let ada_func = FungsiBawaanVM {
        nama: "ada".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'ada' membutuhkan 1 argumen: path".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let path = ctx.get_heap_mut().get_string(*idx).clone();
                let exists = std::path::Path::new(&path).exists();
                Ok(Value::Boolean(exists))
            } else {
                Err("Path harus berupa teks".to_string())
            }
        },
    };
    let ada_idx = vm.heap.alloc(HeapData::FungsiBawaan(ada_func));
    module_dict.insert("ada".to_string(), Value::FungsiBawaan(ada_idx));

    let pindah_func = FungsiBawaanVM {
        nama: "pindah".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err(
                    "Fungsi 'pindah' membutuhkan 2 argumen: path_asal dan path_tujuan".to_string(),
                );
            }
            if let (Value::String(asal_idx), Value::String(tujuan_idx)) = (&args[0], &args[1]) {
                let asal = ctx.get_heap_mut().get_string(*asal_idx).clone();
                let tujuan = ctx.get_heap_mut().get_string(*tujuan_idx).clone();
                match fs::rename(&asal, &tujuan) {
                    Ok(_) => Ok(Value::Kosong),
                    Err(e) => Err(format!(
                        "Gagal memindahkan file dari '{}' ke '{}': {}",
                        asal, tujuan, e
                    )),
                }
            } else {
                Err("Path asal dan tujuan harus berupa teks".to_string())
            }
        },
    };
    let pindah_idx = vm.heap.alloc(HeapData::FungsiBawaan(pindah_func));
    module_dict.insert("pindah".to_string(), Value::FungsiBawaan(pindah_idx));

    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'hapus' membutuhkan 1 argumen: path".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let path = ctx.get_heap_mut().get_string(*idx).clone();
                match fs::remove_file(&path) {
                    Ok(_) => Ok(Value::Kosong),
                    Err(e) => Err(format!("Gagal menghapus file '{}': {}", path, e)),
                }
            } else {
                Err("Path harus berupa teks".to_string())
            }
        },
    };
    let hapus_idx = vm.heap.alloc(HeapData::FungsiBawaan(hapus_func));
    module_dict.insert("hapus".to_string(), Value::FungsiBawaan(hapus_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("file".to_string(), Value::Kamus(dict_idx));
}

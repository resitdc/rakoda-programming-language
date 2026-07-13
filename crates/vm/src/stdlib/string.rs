use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let panjang_func = FungsiBawaanVM {
        nama: "panjang".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'panjang' membutuhkan 1 argumen: teks".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let s = ctx.get_heap_mut().get_string(*idx).clone();
                Ok(Value::Angka(s.len() as f64))
            } else {
                Err("Argumen harus berupa teks".to_string())
            }
        },
    };
    let panjang_idx = vm.heap.alloc(HeapData::FungsiBawaan(panjang_func));
    module_dict.insert("panjang".to_string(), Value::FungsiBawaan(panjang_idx));

    let besar_func = FungsiBawaanVM {
        nama: "besar".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'besar' membutuhkan 1 argumen: teks".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let s = ctx.get_heap_mut().get_string(*idx).clone().to_uppercase();
                let new_idx = ctx.get_heap_mut().alloc(HeapData::String(s));
                Ok(Value::String(new_idx))
            } else {
                Err("Argumen harus berupa teks".to_string())
            }
        },
    };
    let besar_idx = vm.heap.alloc(HeapData::FungsiBawaan(besar_func));
    module_dict.insert("besar".to_string(), Value::FungsiBawaan(besar_idx));

    let kecil_func = FungsiBawaanVM {
        nama: "kecil".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'kecil' membutuhkan 1 argumen: teks".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let s = ctx.get_heap_mut().get_string(*idx).clone().to_lowercase();
                let new_idx = ctx.get_heap_mut().alloc(HeapData::String(s));
                Ok(Value::String(new_idx))
            } else {
                Err("Argumen harus berupa teks".to_string())
            }
        },
    };
    let kecil_idx = vm.heap.alloc(HeapData::FungsiBawaan(kecil_func));
    module_dict.insert("kecil".to_string(), Value::FungsiBawaan(kecil_idx));

    let potong_func = FungsiBawaanVM {
        nama: "potong".to_string(),
        func: |ctx, args| {
            if args.len() != 3 {
                return Err(
                    "Fungsi 'potong' membutuhkan 3 argumen: teks, mulai, selesai".to_string(),
                );
            }
            if let (Value::String(idx), Value::Angka(mulai), Value::Angka(selesai)) =
                (&args[0], &args[1], &args[2])
            {
                let s = ctx.get_heap_mut().get_string(*idx).clone();
                let mulai_idx = *mulai as usize;
                let selesai_idx = *selesai as usize;
                if mulai_idx <= s.len() && selesai_idx <= s.len() && mulai_idx <= selesai_idx {
                    let new_s = s[mulai_idx..selesai_idx].to_string();
                    let new_idx = ctx.get_heap_mut().alloc(HeapData::String(new_s));
                    Ok(Value::String(new_idx))
                } else {
                    Err("Indeks pemotongan di luar batas teks".to_string())
                }
            } else {
                Err("Argumen harus berupa (teks, angka, angka)".to_string())
            }
        },
    };
    let potong_idx = vm.heap.alloc(HeapData::FungsiBawaan(potong_func));
    module_dict.insert("potong".to_string(), Value::FungsiBawaan(potong_idx));

    let ganti_func = FungsiBawaanVM {
        nama: "ganti".to_string(),
        func: |ctx, args| {
            if args.len() != 3 {
                return Err("Fungsi 'ganti' membutuhkan 3 argumen: teks, cari, ganti".to_string());
            }
            if let (Value::String(t_idx), Value::String(c_idx), Value::String(g_idx)) =
                (&args[0], &args[1], &args[2])
            {
                let teks = ctx.get_heap_mut().get_string(*t_idx).clone();
                let cari = ctx.get_heap_mut().get_string(*c_idx).clone();
                let ganti = ctx.get_heap_mut().get_string(*g_idx).clone();
                let new_s = teks.replace(&cari, &ganti);
                let new_idx = ctx.get_heap_mut().alloc(HeapData::String(new_s));
                Ok(Value::String(new_idx))
            } else {
                Err("Argumen harus berupa (teks, teks, teks)".to_string())
            }
        },
    };
    let ganti_idx = vm.heap.alloc(HeapData::FungsiBawaan(ganti_func));
    module_dict.insert("ganti".to_string(), Value::FungsiBawaan(ganti_idx));

    let dari_func = FungsiBawaanVM {
        nama: "dari".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'dari' membutuhkan 1 argumen".to_string());
            }
            let s = args[0].to_string(ctx.get_heap_mut());
            let new_idx = ctx.get_heap_mut().alloc(HeapData::String(s));
            Ok(Value::String(new_idx))
        },
    };
    let dari_idx = vm.heap.alloc(HeapData::FungsiBawaan(dari_func));
    module_dict.insert("dari".to_string(), Value::FungsiBawaan(dari_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("string".to_string(), Value::Kamus(dict_idx));
}

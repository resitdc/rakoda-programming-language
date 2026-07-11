use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::collections::HashMap;
use crate::heap::HeapData;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    let tambah_func = FungsiBawaanVM {
        nama: "tambah".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'tambah' membutuhkan 2 argumen: array dan elemen".to_string());
            }
            if let Value::Array(idx) = &args[0] {
                let elemen = args[1];
                ctx.get_heap_mut().get_array_mut(*idx).push(elemen);
                Ok(Value::Kosong)
            } else {
                Err("Argumen pertama harus berupa array".to_string())
            }
        },
    };
    let tambah_idx = vm.heap.alloc(HeapData::FungsiBawaan(tambah_func));
    module_dict.insert("tambah".to_string(), Value::FungsiBawaan(tambah_idx));

    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'hapus' membutuhkan 2 argumen: array dan indeks".to_string());
            }
            if let (Value::Array(idx), Value::Angka(index)) = (&args[0], &args[1]) {
                let arr = ctx.get_heap_mut().get_array_mut(*idx);
                let i = *index as usize;
                if i < arr.len() {
                    let val = arr.remove(i);
                    Ok(val)
                } else {
                    Err(format!("Indeks {} di luar batas array (panjang {})", i, arr.len()))
                }
            } else {
                Err("Argumen pertama harus array dan kedua harus angka".to_string())
            }
        },
    };
    let hapus_idx = vm.heap.alloc(HeapData::FungsiBawaan(hapus_func));
    module_dict.insert("hapus".to_string(), Value::FungsiBawaan(hapus_idx));

    let panjang_func = FungsiBawaanVM {
        nama: "panjang".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'panjang' membutuhkan 1 argumen: array".to_string());
            }
            if let Value::Array(idx) = &args[0] {
                let len = ctx.get_heap_mut().get_array(*idx).len();
                Ok(Value::Angka(len as f64))
            } else {
                Err("Argumen harus berupa array".to_string())
            }
        },
    };
    let panjang_idx = vm.heap.alloc(HeapData::FungsiBawaan(panjang_func));
    module_dict.insert("panjang".to_string(), Value::FungsiBawaan(panjang_idx));

    let ambil_func = FungsiBawaanVM {
        nama: "ambil".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'ambil' membutuhkan 2 argumen: array dan indeks".to_string());
            }
            if let (Value::Array(idx), Value::Angka(index)) = (&args[0], &args[1]) {
                let arr = ctx.get_heap_mut().get_array_mut(*idx);
                let i = *index as usize;
                if i < arr.len() {
                    Ok(arr[i])
                } else {
                    Err(format!("Indeks {} di luar batas array (panjang {})", i, arr.len()))
                }
            } else {
                Err("Argumen harus berupa array dan angka".to_string())
            }
        },
    };
    let ambil_idx = vm.heap.alloc(HeapData::FungsiBawaan(ambil_func));
    module_dict.insert("ambil".to_string(), Value::FungsiBawaan(ambil_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("list".to_string(), Value::Kamus(dict_idx));
}

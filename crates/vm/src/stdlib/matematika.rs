use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::collections::HashMap;
use crate::heap::{HeapData};
use rand::Rng;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    let tambah_func = FungsiBawaanVM {
        nama: "tambah".to_string(),
        func: |_heap, args| {
            if args.len() == 2 {
                if let (Value::Angka(a), Value::Angka(b)) = (&args[0], &args[1]) {
                    return Ok(Value::Angka(a + b));
                }
            }
            Ok(Value::Kosong)
        },
    };
    let tambah_idx = vm.heap.alloc(HeapData::FungsiBawaan(tambah_func));
    module_dict.insert("tambah".to_string(), Value::FungsiBawaan(tambah_idx));

    let kurang_func = FungsiBawaanVM {
        nama: "kurang".to_string(),
        func: |_heap, args| {
            if args.len() == 2 {
                if let (Value::Angka(a), Value::Angka(b)) = (&args[0], &args[1]) {
                    return Ok(Value::Angka(a - b));
                }
            }
            Ok(Value::Kosong)
        },
    };
    let kurang_idx = vm.heap.alloc(HeapData::FungsiBawaan(kurang_func));
    module_dict.insert("kurang".to_string(), Value::FungsiBawaan(kurang_idx));

    let kali_func = FungsiBawaanVM {
        nama: "kali".to_string(),
        func: |_heap, args| {
            if args.len() == 2 {
                if let (Value::Angka(a), Value::Angka(b)) = (&args[0], &args[1]) {
                    return Ok(Value::Angka(a * b));
                }
            }
            Ok(Value::Kosong)
        },
    };
    let kali_idx = vm.heap.alloc(HeapData::FungsiBawaan(kali_func));
    module_dict.insert("kali".to_string(), Value::FungsiBawaan(kali_idx));

    let bagi_func = FungsiBawaanVM {
        nama: "bagi".to_string(),
        func: |_heap, args| {
            if args.len() == 2 {
                if let (Value::Angka(a), Value::Angka(b)) = (&args[0], &args[1]) {
                    if *b != 0.0 {
                        return Ok(Value::Angka(a / b));
                    }
                }
            }
            Ok(Value::Kosong)
        },
    };
    let bagi_idx = vm.heap.alloc(HeapData::FungsiBawaan(bagi_func));
    module_dict.insert("bagi".to_string(), Value::FungsiBawaan(bagi_idx));

    let pangkat_func = FungsiBawaanVM {
        nama: "pangkat".to_string(),
        func: |_heap, args| {
            if args.len() == 2 {
                if let (Value::Angka(a), Value::Angka(b)) = (&args[0], &args[1]) {
                    return Ok(Value::Angka(a.powf(*b)));
                }
            }
            Ok(Value::Kosong)
        },
    };
    let pangkat_idx = vm.heap.alloc(HeapData::FungsiBawaan(pangkat_func));
    module_dict.insert("pangkat".to_string(), Value::FungsiBawaan(pangkat_idx));

    let acak_func = FungsiBawaanVM {
        nama: "acak".to_string(),
        func: |_heap, args| {
            let mut rng = rand::thread_rng();
            if args.len() == 2 {
                if let (Value::Angka(min), Value::Angka(max)) = (&args[0], &args[1]) {
                    if max > min {
                        let acak: f64 = rng.gen_range(*min..*max);
                        return Ok(Value::Angka(acak));
                    }
                }
            }
            Ok(Value::Angka(rng.gen_range(0.0..1.0)))
        },
    };
    let acak_idx = vm.heap.alloc(HeapData::FungsiBawaan(acak_func));
    module_dict.insert("acak".to_string(), Value::FungsiBawaan(acak_idx));

    let bulat_func = FungsiBawaanVM {
        nama: "bulatkan".to_string(),
        func: |_heap, args| {
            if args.len() == 1 {
                if let Value::Angka(n) = args[0] {
                    return Ok(Value::Angka(n.round()));
                }
            }
            Ok(Value::Kosong)
        },
    };
    let bulat_idx = vm.heap.alloc(HeapData::FungsiBawaan(bulat_func));
    module_dict.insert("bulatkan".to_string(), Value::FungsiBawaan(bulat_idx));

    let akar_func = FungsiBawaanVM {
        nama: "akar".to_string(),
        func: |_heap, args| {
            if args.len() == 1 {
                if let Value::Angka(n) = args[0] {
                    if n >= 0.0 {
                        return Ok(Value::Angka(n.sqrt()));
                    }
                }
            }
            Ok(Value::Kosong)
        },
    };
    let akar_idx = vm.heap.alloc(HeapData::FungsiBawaan(akar_func));
    module_dict.insert("akar".to_string(), Value::FungsiBawaan(akar_idx));

    let kuadrat_func = FungsiBawaanVM {
        nama: "kuadrat".to_string(),
        func: |_heap, args| {
            if args.len() == 1 {
                if let Value::Angka(n) = args[0] {
                    return Ok(Value::Angka(n * n));
                }
            }
            Ok(Value::Kosong)
        },
    };
    let kuadrat_idx = vm.heap.alloc(HeapData::FungsiBawaan(kuadrat_func));
    module_dict.insert("kuadrat".to_string(), Value::FungsiBawaan(kuadrat_idx));
    
    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("matematika".to_string(), Value::Kamus(dict_idx));
}

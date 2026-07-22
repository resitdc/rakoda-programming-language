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
    for (nama, func_ref) in &stdlib::core::fungsi_core() {
        // Skip tampilkan and baca — they need I/O wrappers
        if *nama == "tampilkan" || *nama == "baca" {
            continue;
        }
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
        func: std::sync::Arc::new(tampilkan_wrapper),
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
        func: std::sync::Arc::new(baca_wrapper),
    };
    let baca_idx = vm.heap.alloc(HeapData::FungsiBawaan(baca));
    module_dict.insert("baca".to_string(), Value::FungsiBawaan(baca_idx));

    let module_idx = vm.heap.alloc(HeapData::Modul(module_dict));
    vm.set_global("core".to_string(), Value::Modul(module_idx));

    // uuid
    fn uuid_wrapper(ctx: &mut dyn VmContext, _args: Vec<Value>) -> Result<Value, String> {
        let u = uuid::Uuid::new_v4().to_string();
        let idx = ctx.get_heap_mut().alloc(HeapData::String(u));
        Ok(Value::String(idx))
    }
    let uuid_func = FungsiBawaanVM {
        nama: "uuid".to_string(),
        func: std::sync::Arc::new(uuid_wrapper),
    };
    let uuid_idx = vm.heap.alloc(HeapData::FungsiBawaan(uuid_func));
    vm.set_global("uuid".to_string(), Value::FungsiBawaan(uuid_idx));

    // acak
    fn acak_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if args.len() == 2 {
            if let (Value::Angka(min), Value::Angka(max)) = (args[0], args[1]) {
                if min > max {
                    return Err("acak: nilai minimum tidak boleh lebih besar dari maksimum".to_string());
                }
                let is_integer = min.fract() == 0.0 && max.fract() == 0.0;
                let res = if is_integer {
                    rng.gen_range((min as i64)..=(max as i64)) as f64
                } else {
                    rng.gen_range(min..=max)
                };
                return Ok(Value::Angka(res));
            }
            
            if let (Value::String(tipe_idx), Value::Angka(len)) = (args[0], args[1]) {
                let tipe = ctx.get_heap_mut().get_string(tipe_idx).clone();
                let length = len as usize;
                let chars: Vec<char> = match tipe.as_str() {
                    "huruf" => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
                    "angka" => "0123456789".chars().collect(),
                    "campuran" => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect(),
                    _ => return Err("acak: tipe tidak valid. Gunakan 'huruf', 'angka', atau 'campuran'".to_string()),
                };
                let result: String = (0..length)
                    .map(|_| {
                        let idx = rng.gen_range(0..chars.len());
                        chars[idx]
                    })
                    .collect();
                let res_idx = ctx.get_heap_mut().alloc(HeapData::String(result));
                return Ok(Value::String(res_idx));
            }
        } else if args.len() == 1 {
            if let Value::String(tipe_idx) = args[0] {
                let tipe = ctx.get_heap_mut().get_string(tipe_idx).clone();
                let hasil = match tipe.as_str() {
                    "nama" => {
                        let depan = ["Restu", "Salwa", "Bernandus", "Zidane", "Icksan", "Aji", "Aam", "Ilham", "Babaw", "Revin", "Teguh", "Hotma", "Brian", "Abim", "Encep", "Cae", "Cynthia", "Iza", "Gusti", "Ridho"];
                        let belakang = ["Dwi Cahyo", "Nugraha", "Silaen", "Capah", "Aji", "Naibaho", "Fathur", "Reginal", "Agung", "Akmal", "Ihwan", "Azkia", "Rahayu", "Novia", "Arahman", "Al Sadawi"];
                        format!("{} {}", depan[rng.gen_range(0..depan.len())], belakang[rng.gen_range(0..belakang.len())])
                    }
                    "alamat" => {
                        let jalan = ["Jl. Sadang Serang", "Jl. Thamrin", "Jl. Melati", "Jl. Mawar", "Jl. Diponegoro", "Jl. Merdeka", "Jl. Gatot Subroto", "Jl. Pahlawan"];
                        let kota = ["Jakarta", "Surabaya", "Bandung", "Medan", "Semarang", "Makassar", "Palembang", "Denpasar", "Yogyakarta", "Malang"];
                        format!("{} No. {}, {}", jalan[rng.gen_range(0..jalan.len())], rng.gen_range(1..100), kota[rng.gen_range(0..kota.len())])
                    }
                    "kota" => {
                        let kota = ["Jakarta", "Surabaya", "Bandung", "Medan", "Semarang", "Makassar", "Palembang", "Denpasar", "Yogyakarta", "Malang", "Balikpapan", "Samarinda", "Banjarmasin"];
                        kota[rng.gen_range(0..kota.len())].to_string()
                    }
                    "telepon" => {
                        format!("0812-{:04}-{:04}", rng.gen_range(1000..9999), rng.gen_range(1000..9999))
                    }
                    _ => return Err(format!("acak: argumen '{}' tidak didukung untuk satu parameter", tipe)),
                };
                let res_idx = ctx.get_heap_mut().alloc(HeapData::String(hasil));
                return Ok(Value::String(res_idx));
            }
        } else if args.is_empty() {
            let res: f64 = rng.gen_range(0.0..1.0);
            return Ok(Value::Angka(res));
        }

        Err("acak: argumen tidak valid. Gunakan acak(min, max), acak('tipe', panjang), atau acak('tipe_faker')".to_string())
    }
    let acak_func = FungsiBawaanVM {
        nama: "acak".to_string(),
        func: std::sync::Arc::new(acak_wrapper),
    };
    let acak_idx = vm.heap.alloc(HeapData::FungsiBawaan(acak_func));
    vm.set_global("acak".to_string(), Value::FungsiBawaan(acak_idx));
    vm.set_global("random".to_string(), Value::FungsiBawaan(acak_idx));
}

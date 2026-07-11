use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;

pub fn register(vm: &mut VM) {
    let angka_func = FungsiBawaanVM {
        nama: "angka".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("Fungsi 'angka' membutuhkan 1 argumen".to_string());
            }
            match &args[0] {
                Value::Angka(n) => Ok(Value::Angka(*n)),
                Value::String(s) => {
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
    
    vm.set_global("angka".to_string(), Value::FungsiBawaan(Rc::new(angka_func)));

    let teks_func = FungsiBawaanVM {
        nama: "teks".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("Fungsi 'teks' membutuhkan 1 argumen".to_string());
            }
            Ok(Value::String(Rc::new(args[0].to_string())))
        },
    };
    
    vm.set_global("teks".to_string(), Value::FungsiBawaan(Rc::new(teks_func)));
}

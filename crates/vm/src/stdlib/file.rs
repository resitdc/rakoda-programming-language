use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fs;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    // file.baca(path)
    let baca_func = FungsiBawaanVM {
        nama: "baca".to_string(),
        func: |args| {
            if args.is_empty() {
                return Err("Fungsi 'baca' membutuhkan 1 argumen: path".to_string());
            }
            if let Value::String(path) = &args[0] {
                match fs::read_to_string(path.as_ref()) {
                    Ok(content) => Ok(Value::String(Rc::new(content))),
                    Err(e) => Err(format!("Gagal membaca file '{}': {}", path, e)),
                }
            } else {
                Err("Path harus berupa teks".to_string())
            }
        },
    };
    module_dict.insert("baca".to_string(), Value::FungsiBawaan(Rc::new(baca_func)));

    // file.tulis(path, content)
    let tulis_func = FungsiBawaanVM {
        nama: "tulis".to_string(),
        func: |args| {
            if args.len() != 2 {
                return Err("Fungsi 'tulis' membutuhkan 2 argumen: path dan isi".to_string());
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match fs::write(path.as_ref(), content.as_bytes()) {
                    Ok(_) => Ok(Value::Kosong),
                    Err(e) => Err(format!("Gagal menulis ke file '{}': {}", path, e)),
                }
            } else {
                Err("Path dan isi harus berupa teks".to_string())
            }
        },
    };
    module_dict.insert("tulis".to_string(), Value::FungsiBawaan(Rc::new(tulis_func)));

    vm.set_global("file".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

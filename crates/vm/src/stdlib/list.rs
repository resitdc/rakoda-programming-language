use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    // list.tambah(data, nilai)
    let tambah_func = FungsiBawaanVM {
        nama: "tambah".to_string(),
        func: |args| {
            if args.len() != 2 {
                return Err("Fungsi 'tambah' membutuhkan 2 argumen: array dan nilai".to_string());
            }
            if let Value::Array(arr) = &args[0] {
                arr.borrow_mut().push(args[1].clone());
                Ok(Value::Kosong)
            } else {
                Err("Argumen pertama harus berupa array".to_string())
            }
        },
    };
    module_dict.insert("tambah".to_string(), Value::FungsiBawaan(Rc::new(tambah_func)));

    // list.hapus(data, indeks)
    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: |args| {
            if args.len() != 2 {
                return Err("Fungsi 'hapus' membutuhkan 2 argumen: array dan indeks".to_string());
            }
            if let (Value::Array(arr), Value::Angka(idx)) = (&args[0], &args[1]) {
                let mut borrowed_arr = arr.borrow_mut();
                let index = *idx as usize;
                if index < borrowed_arr.len() {
                    borrowed_arr.remove(index);
                }
                Ok(Value::Kosong)
            } else {
                Err("Argumen pertama harus array dan argumen kedua harus angka".to_string())
            }
        },
    };
    module_dict.insert("hapus".to_string(), Value::FungsiBawaan(Rc::new(hapus_func)));

    // list.panjang(data)
    let panjang_func = FungsiBawaanVM {
        nama: "panjang".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("Fungsi 'panjang' membutuhkan 1 argumen: array".to_string());
            }
            if let Value::Array(arr) = &args[0] {
                Ok(Value::Angka(arr.borrow().len() as f64))
            } else {
                Err("Argumen harus berupa array".to_string())
            }
        },
    };
    module_dict.insert("panjang".to_string(), Value::FungsiBawaan(Rc::new(panjang_func)));

    vm.set_global("list".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

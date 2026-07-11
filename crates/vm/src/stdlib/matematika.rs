use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    let kuadrat_func = FungsiBawaanVM {
        nama: "kuadrat".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("Fungsi 'kuadrat' membutuhkan 1 argumen".to_string());
            }
            if let Value::Angka(n) = args[0] {
                Ok(Value::Angka(n * n))
            } else {
                Err("Argumen harus berupa angka".to_string())
            }
        },
    };
    
    module_dict.insert("kuadrat".to_string(), Value::FungsiBawaan(Rc::new(kuadrat_func)));

    vm.set_global("matematika".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

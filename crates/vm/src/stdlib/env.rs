use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::env;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    // Attempt to load .env file silently. It's okay if it fails (e.g. file doesn't exist).
    let _ = dotenvy::dotenv();
    
    // env.get(key)
    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |args| {
            if args.is_empty() {
                return Err("Fungsi 'get' membutuhkan 1 argumen: kunci (key)".to_string());
            }
            if let Value::String(key) = &args[0] {
                match env::var(key.as_ref()) {
                    Ok(val) => Ok(Value::String(Rc::new(val))),
                    Err(_) => Ok(Value::Kosong),
                }
            } else {
                Err("Kunci (key) harus berupa teks".to_string())
            }
        },
    };
    module_dict.insert("get".to_string(), Value::FungsiBawaan(Rc::new(get_func)));

    vm.set_global("env".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

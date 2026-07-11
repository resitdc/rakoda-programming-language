use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    // http.get(url)
    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |args| {
            if args.is_empty() {
                return Err("Fungsi 'get' membutuhkan minimal 1 argumen: url".to_string());
            }
            if let Value::String(url) = &args[0] {
                match ureq::get(url.as_ref()).call() {
                    Ok(mut response) => {
                        let mut resp_dict = HashMap::new();
                        resp_dict.insert("status".to_string(), Value::Angka(response.status().as_u16() as f64));
                        
                        let body = response.body_mut().read_to_string().unwrap_or_default();
                        resp_dict.insert("body".to_string(), Value::String(Rc::new(body)));
                        
                        Ok(Value::Kamus(Rc::new(RefCell::new(resp_dict))))
                    }
                    Err(e) => Err(format!("Permintaan HTTP gagal: {}", e)),
                }
            } else {
                Err("URL harus berupa teks".to_string())
            }
        },
    };
    module_dict.insert("get".to_string(), Value::FungsiBawaan(Rc::new(get_func)));

    vm.set_global("http".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

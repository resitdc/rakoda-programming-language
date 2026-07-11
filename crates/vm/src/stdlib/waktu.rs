use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::cell::RefCell;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();
    
    let sekarang_func = FungsiBawaanVM {
        nama: "sekarang".to_string(),
        func: |_args| {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .unwrap();
            Ok(Value::Angka(since_the_epoch.as_secs_f64() * 1000.0)) // milliseconds
        },
    };
    
    module_dict.insert("sekarang".to_string(), Value::FungsiBawaan(Rc::new(sekarang_func)));

    vm.set_global("waktu".to_string(), Value::Kamus(Rc::new(RefCell::new(module_dict))));
}

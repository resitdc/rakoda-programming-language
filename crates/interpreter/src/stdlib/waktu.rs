//! Thin adapter: wraps stdlib crate's waktu module for interpreter use.
use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use crate::stdlib::adapter::bungkus_fungsi;
use std::cell::RefCell;
use std::rc::Rc;
use stdlib;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    for (nama, f) in stdlib::fungsi_waktu() {
        module_env
            .borrow_mut()
            .set(nama.to_string(), Objek::MetodeBawaan(bungkus_fungsi(f)));
    }

    env.borrow_mut()
        .set("waktu".to_string(), Objek::Modul(module_env));
}

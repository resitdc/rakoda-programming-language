use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use md5::{Digest as Md5Digest, Md5};
use sha2::Sha256;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    module_env.borrow_mut().set(
        "md5".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 1
                && let Objek::String(s) = &args[0] {
                    let mut hasher = Md5::new();
                    hasher.update(s.as_bytes());
                    let result = hasher.finalize();
                    return Objek::String(format!("{:x}", result));
                }
            Objek::Kosong
        }),
    );

    module_env.borrow_mut().set(
        "sha256".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 1
                && let Objek::String(s) = &args[0] {
                    let mut hasher = Sha256::new();
                    hasher.update(s.as_bytes());
                    let result = hasher.finalize();
                    return Objek::String(format!("{:x}", result));
                }
            Objek::Kosong
        }),
    );

    env.borrow_mut()
        .set("kripto".to_string(), Objek::Modul(module_env));
}

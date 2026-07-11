use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use crate::objek::Objek;
use crate::lingkungan::Lingkungan;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();
    
    // file.tulis("catatan.txt", "Belajar RPL")
    module_env.borrow_mut().set("tulis".to_string(), Objek::FungsiBawaan(|args| {
        if args.len() == 2 {
            if let (Objek::String(nama_file), Objek::String(isi)) = (&args[0], &args[1]) {
                if fs::write(nama_file, isi).is_ok() {
                    return Objek::Boolean(true);
                }
            }
        }
        Objek::Boolean(false)
    }));

    // file.baca("catatan.txt")
    module_env.borrow_mut().set("baca".to_string(), Objek::FungsiBawaan(|args| {
        if args.len() == 1 {
            if let Objek::String(nama_file) = &args[0] {
                if let Ok(isi) = fs::read_to_string(nama_file) {
                    return Objek::String(isi);
                }
            }
        }
        Objek::Kosong
    }));
    
    // file.ada("catatan.txt") -> check existence
    module_env.borrow_mut().set("ada".to_string(), Objek::FungsiBawaan(|args| {
        if args.len() == 1 {
            if let Objek::String(nama_file) = &args[0] {
                return Objek::Boolean(Path::new(nama_file).exists());
            }
        }
        Objek::Boolean(false)
    }));

    env.borrow_mut().set("file".to_string(), Objek::Modul(module_env));
}

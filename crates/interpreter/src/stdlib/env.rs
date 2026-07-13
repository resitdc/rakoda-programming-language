use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let mut env_map = HashMap::new();

    // env.get("KEY")
    env_map.insert(
        "get".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(Objek::String(kunci)) = args.first() {
                match std::env::var(kunci) {
                    Ok(val) => Objek::String(val),
                    Err(_) => Objek::Kosong,
                }
            } else {
                Objek::Kosong
            }
        }),
    );

    // env.set("KEY", "VALUE")
    env_map.insert(
        "set".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() >= 2
                && let (Objek::String(kunci), Objek::String(nilai)) = (&args[0], &args[1]) {
                    unsafe {
                        std::env::set_var(kunci, nilai);
                    }
                    return Objek::Boolean(true);
                }
            Objek::Boolean(false)
        }),
    );

    // env.load() atau env.load(".env")
    env_map.insert(
        "load".to_string(),
        Objek::FungsiBawaan(|args| {
            let result = if args.is_empty() {
                dotenvy::dotenv()
            } else if let Some(Objek::String(path)) = args.first() {
                dotenvy::from_filename(path)
            } else {
                return Objek::Boolean(false);
            };

            match result {
                Ok(_) => Objek::Boolean(true),
                Err(_) => Objek::Boolean(false),
            }
        }),
    );

    let env_objek = Objek::Kamus(Rc::new(RefCell::new(env_map)));
    env.borrow_mut().set("env".to_string(), env_objek);
}

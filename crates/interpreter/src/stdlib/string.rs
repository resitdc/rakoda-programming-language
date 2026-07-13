use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    // string.panjang(teks)
    module_env.borrow_mut().set(
        "panjang".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(Objek::String(s)) = args.first() {
                return Objek::Angka(s.len() as f64);
            }
            Objek::Kosong
        }),
    );

    // string.besar(teks)
    module_env.borrow_mut().set(
        "besar".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(Objek::String(s)) = args.first() {
                return Objek::String(s.to_uppercase());
            }
            Objek::Kosong
        }),
    );

    // string.kecil(teks)
    module_env.borrow_mut().set(
        "kecil".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(Objek::String(s)) = args.first() {
                return Objek::String(s.to_lowercase());
            }
            Objek::Kosong
        }),
    );

    // string.potong(teks, start, end)
    module_env.borrow_mut().set(
        "potong".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() >= 3
                && let (Objek::String(s), Objek::Angka(mulai), Objek::Angka(selesai)) =
                    (&args[0], &args[1], &args[2])
                {
                    let mulai = *mulai as usize;
                    let selesai = *selesai as usize;
                    if mulai <= s.len() && selesai <= s.len() && mulai <= selesai {
                        return Objek::String(s[mulai..selesai].to_string());
                    }
                }
            Objek::Kosong
        }),
    );

    // string.ganti("Halo Budi", "Budi", "Ani")
    module_env.borrow_mut().set(
        "ganti".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 3
                && let (Objek::String(s), Objek::String(cari), Objek::String(ganti)) =
                    (&args[0], &args[1], &args[2])
                {
                    return Objek::String(s.replace(cari, ganti));
                }
            Objek::Kosong
        }),
    );

    env.borrow_mut()
        .set("string".to_string(), Objek::Modul(module_env));
}

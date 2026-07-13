use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use rand::RngExt;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    // matematika.tambah
    module_env.borrow_mut().set(
        "tambah".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Angka(a), Objek::Angka(b)) = (&args[0], &args[1]) {
                    return Objek::Angka(a + b);
                }
            Objek::Kosong
        }),
    );

    // matematika.kurang
    module_env.borrow_mut().set(
        "kurang".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Angka(a), Objek::Angka(b)) = (&args[0], &args[1]) {
                    return Objek::Angka(a - b);
                }
            Objek::Kosong
        }),
    );

    // matematika.kali
    module_env.borrow_mut().set(
        "kali".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Angka(a), Objek::Angka(b)) = (&args[0], &args[1]) {
                    return Objek::Angka(a * b);
                }
            Objek::Kosong
        }),
    );

    // matematika.bagi
    module_env.borrow_mut().set(
        "bagi".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Angka(a), Objek::Angka(b)) = (&args[0], &args[1])
                    && *b != 0.0 {
                        return Objek::Angka(a / b);
                    }
            Objek::Kosong
        }),
    );

    // matematika.pangkat
    module_env.borrow_mut().set(
        "pangkat".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Angka(a), Objek::Angka(b)) = (&args[0], &args[1]) {
                    return Objek::Angka(a.powf(*b));
                }
            Objek::Kosong
        }),
    );

    // matematika.bulatkan
    module_env.borrow_mut().set(
        "bulatkan".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(Objek::Angka(a)) = args.first() {
                return Objek::Angka(a.round());
            }
            Objek::Kosong
        }),
    );

    // matematika.acak
    module_env.borrow_mut().set(
        "acak".to_string(),
        Objek::FungsiBawaan(|args| {
            let mut rng = rand::rng();
            if args.len() == 2
                && let (Objek::Angka(min), Objek::Angka(max)) = (&args[0], &args[1])
                    && max > min {
                        let acak: f64 = rng.random_range(*min..*max);
                        return Objek::Angka(acak);
                    }
            let acak: f64 = rng.random_range(0.0..1.0);
            Objek::Angka(acak)
        }),
    );

    env.borrow_mut()
        .set("matematika".to_string(), Objek::Modul(module_env));
}

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use chrono::{Datelike, Local, Timelike};
use std::cell::RefCell;
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    // waktu.sekarang()
    module_env.borrow_mut().set(
        "sekarang".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().timestamp() as f64)),
    );

    // waktu.tahun()
    module_env.borrow_mut().set(
        "tahun".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().year() as f64)),
    );

    // waktu.bulan()
    module_env.borrow_mut().set(
        "bulan".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().month() as f64)),
    );

    // waktu.tanggal()
    module_env.borrow_mut().set(
        "tanggal".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().day() as f64)),
    );

    // waktu.jam()
    module_env.borrow_mut().set(
        "jam".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().hour() as f64)),
    );

    // waktu.menit()
    module_env.borrow_mut().set(
        "menit".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().minute() as f64)),
    );

    // waktu.detik()
    module_env.borrow_mut().set(
        "detik".to_string(),
        Objek::FungsiBawaan(|_args| Objek::Angka(Local::now().second() as f64)),
    );

    // waktu.format()
    module_env.borrow_mut().set(
        "format".to_string(),
        Objek::FungsiBawaan(|_args| {
            Objek::String(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
        }),
    );

    env.borrow_mut()
        .set("waktu".to_string(), Objek::Modul(module_env));
}

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    let module_env = Lingkungan::baru();

    // list.tambah(data, value)
    module_env.borrow_mut().set(
        "tambah".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let Objek::Array(arr) = &args[0] {
                    arr.borrow_mut().push(args[1].clone());
                    return Objek::Kosong;
                }
            Objek::Kosong
        }),
    );

    // list.hapus(data, index) -> removing by index
    module_env.borrow_mut().set(
        "hapus".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Array(arr), Objek::Angka(idx)) = (&args[0], &args[1]) {
                    let mut borrowed_arr = arr.borrow_mut();
                    let index = *idx as usize;
                    if index < borrowed_arr.len() {
                        borrowed_arr.remove(index);
                    }
                }
            Objek::Kosong
        }),
    );

    // list.panjang(data)
    module_env.borrow_mut().set(
        "panjang".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(Objek::Array(arr)) = args.first() {
                return Objek::Angka(arr.borrow().len() as f64);
            }
            Objek::Kosong
        }),
    );

    // list.ambil(data, index)
    module_env.borrow_mut().set(
        "ambil".to_string(),
        Objek::FungsiBawaan(|args| {
            if args.len() == 2
                && let (Objek::Array(arr), Objek::Angka(idx)) = (&args[0], &args[1]) {
                    let borrowed_arr = arr.borrow();
                    let index = *idx as usize;
                    if index < borrowed_arr.len() {
                        return borrowed_arr[index].clone();
                    }
                }
            Objek::Kosong
        }),
    );

    env.borrow_mut()
        .set("list".to_string(), Objek::Modul(module_env));
}

//! Adapter untuk konversi antara Objek (interpreter) dan NilaiRpl (stdlib unifikasi).
//! Thin layer — tidak mengandung logika bisnis stdlib.

use crate::objek::Objek;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use stdlib::jenis::NilaiRpl;

/// Konversi Objek -> NilaiRpl
pub fn objek_ke_nilai(obj: &Objek) -> NilaiRpl {
    match obj {
        Objek::Angka(n) => NilaiRpl::Angka(*n),
        Objek::String(s) => NilaiRpl::Teks(s.clone()),
        Objek::Boolean(b) => NilaiRpl::Boolean(*b),
        Objek::Kosong => NilaiRpl::Kosong,
        Objek::Array(arr) => {
            let items: Vec<NilaiRpl> = arr.borrow().iter().map(objek_ke_nilai).collect();
            NilaiRpl::Daftar(items)
        }
        Objek::Kamus(map) => {
            let mut kamus = HashMap::new();
            for (k, v) in map.borrow().iter() {
                kamus.insert(k.clone(), objek_ke_nilai(v));
            }
            NilaiRpl::Kamus(kamus)
        }
        _ => NilaiRpl::Teks(format!("{}", obj)),
    }
}

/// Konversi &[Objek] -> Vec<NilaiRpl>
pub fn objek_slice_ke_nilai_vec(args: &[Objek]) -> Vec<NilaiRpl> {
    args.iter().map(objek_ke_nilai).collect()
}

/// Konversi NilaiRpl -> Objek
pub fn nilai_ke_objek(val: &NilaiRpl) -> Objek {
    match val {
        NilaiRpl::Angka(n) => Objek::Angka(*n),
        NilaiRpl::Teks(s) => Objek::String(s.clone()),
        NilaiRpl::Boolean(b) => Objek::Boolean(*b),
        NilaiRpl::Kosong => Objek::Kosong,
        NilaiRpl::Daftar(items) => {
            let arr: Vec<Objek> = items.iter().map(nilai_ke_objek).collect();
            Objek::Array(Rc::new(RefCell::new(arr)))
        }
        NilaiRpl::Kamus(map) => {
            let mut kamus = HashMap::new();
            for (k, v) in map.iter() {
                kamus.insert(k.clone(), nilai_ke_objek(v));
            }
            Objek::Kamus(Rc::new(RefCell::new(kamus)))
        }
    }
}

/// Bungkus fungsi murni stdlib (fn(&[NilaiRpl]) -> Result<NilaiRpl, String>)
/// menjadi Objek::MetodeBawaan yang kompatibel dengan interpreter.
pub fn bungkus_fungsi(
    f: fn(&[NilaiRpl]) -> Result<NilaiRpl, String>,
) -> Rc<dyn Fn(Vec<Objek>) -> Objek> {
    Rc::new(move |args: Vec<Objek>| {
        let nilai_args: Vec<NilaiRpl> = args.iter().map(objek_ke_nilai).collect();
        match f(&nilai_args) {
            Ok(nilai) => nilai_ke_objek(&nilai),
            Err(pesan) => Objek::Pengecualian(Box::new(Objek::String(pesan))),
        }
    })
}

//! Thin adapter: wraps shared stdlib's core module for interpreter use.
//! Pure functions (angka, teks, boolean) delegate via adapter::bungkus_fungsi.
//! I/O functions (tampilkan, baca) need local wrappers.

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use crate::stdlib::adapter;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    // Pure functions dari shared stdlib: angka, teks, boolean
    for (nama, func) in &stdlib::core::fungsi_core() {
        if *nama == "tampilkan" || *nama == "baca" {
            continue;
        }
        env.borrow_mut().set(
            nama.to_string(),
            Objek::MetodeBawaan(adapter::bungkus_fungsi(*func)),
        );
    }

    // "tampilkan" dengan I/O wrapper (delegasikan formatting ke shared stdlib)
    // Shared stdlib tampilkan_impl returns a formatted string; we print it.
    // But we can't call private impl directly. Reuse the same formatting logic.
    env.borrow_mut().set(
        "tampilkan".to_string(),
        Objek::FungsiBawaan(|args| {
            use stdlib::jenis::NilaiRpl;
            let nilai_args: Vec<NilaiRpl> = args.iter().map(adapter::objek_ke_nilai).collect();
            let output = nilai_args
                .iter()
                .map(|a| match a {
                    NilaiRpl::Teks(s) => s.clone(),
                    NilaiRpl::Angka(n) => n.to_string(),
                    NilaiRpl::Boolean(b) => b.to_string(),
                    NilaiRpl::Kosong => "tidak ada".to_string(),
                    _ => format!("{:?}", a),
                })
                .collect::<Vec<_>>()
                .join("");
            print!("{}", output);
            println!();
            io::stdout().flush().unwrap_or(());
            Objek::Kosong
        }),
    );

    // "baca" dengan I/O wrapper
    env.borrow_mut().set(
        "baca".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt);
                io::stdout().flush().unwrap_or(());
            }
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap_or(0);
            Objek::String(input.trim_end().to_string())
        }),
    );
}

use crate::lingkungan::Lingkungan;
use crate::objek::Objek;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

pub fn register(env: &Rc<RefCell<Lingkungan>>) {
    // fungsi: tampilkan(x)
    env.borrow_mut().set(
        "tampilkan".to_string(),
        Objek::FungsiBawaan(|args| {
            for arg in args {
                print!("{}", arg.to_string_pretty(0, true));
            }
            println!();
            Objek::Kosong
        }),
    );

    // fungsi: baca(prompt)
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

    // fungsi: angka(x)
    env.borrow_mut().set(
        "angka".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(arg) = args.first() {
                match arg {
                    Objek::Angka(n) => Objek::Angka(*n),
                    Objek::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            Objek::Angka(n)
                        } else {
                            Objek::Kosong
                        }
                    }
                    Objek::Boolean(b) => Objek::Angka(if *b { 1.0 } else { 0.0 }),
                    _ => Objek::Kosong,
                }
            } else {
                Objek::Kosong
            }
        }),
    );

    // fungsi: teks(x)
    env.borrow_mut().set(
        "teks".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(arg) = args.first() {
                Objek::String(format!("{}", arg))
            } else {
                Objek::String(String::new())
            }
        }),
    );

    // fungsi: boolean(x)
    env.borrow_mut().set(
        "boolean".to_string(),
        Objek::FungsiBawaan(|args| {
            if let Some(arg) = args.first() {
                match arg {
                    Objek::String(s) => {
                        let s_lower = s.to_lowercase();
                        Objek::Boolean(s_lower == "true" || s_lower == "benar" || s_lower == "1")
                    }
                    Objek::Angka(n) => Objek::Boolean(*n != 0.0),
                    Objek::Boolean(b) => Objek::Boolean(*b),
                    _ => Objek::Boolean(false),
                }
            } else {
                Objek::Boolean(false)
            }
        }),
    );
}

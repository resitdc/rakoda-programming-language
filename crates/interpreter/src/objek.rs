use crate::lingkungan::Lingkungan;
use ast::Statement;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Objek {
    Angka(f64),
    String(String),
    Boolean(bool),
    Kosong,
    Kembalikan(Box<Objek>),
    Pengecualian(Box<Objek>),
    Fungsi {
        parameter: Vec<String>,
        body: Vec<Statement>,
        env: Rc<RefCell<Lingkungan>>,
    },
    FungsiBawaan(fn(Vec<Objek>) -> Objek),
    MetodeBawaan(Rc<dyn Fn(Vec<Objek>) -> Objek>),
    Modul(Rc<RefCell<Lingkungan>>),
    Array(Rc<RefCell<Vec<Objek>>>),
    Kamus(Rc<RefCell<HashMap<String, Objek>>>),
}

impl Objek {
    pub fn to_string_pretty(&self, indent: usize, is_root: bool) -> String {
        let spaces = " ".repeat(indent);
        let inner_spaces = " ".repeat(indent + 2);
        match self {
            Objek::Array(elemen) => {
                let borrow = elemen.borrow();
                if borrow.is_empty() {
                    return "[]".to_string();
                }
                let mut s = String::from("[\n");
                for (i, e) in borrow.iter().enumerate() {
                    s.push_str(&inner_spaces);
                    s.push_str(&e.to_string_pretty(indent + 2, false));
                    if i < borrow.len() - 1 {
                        s.push(',');
                    }
                    s.push('\n');
                }
                s.push_str(&format!("{}]", spaces));
                s
            }
            Objek::Kamus(pasangan) => {
                let borrow = pasangan.borrow();
                if borrow.is_empty() {
                    return "{}".to_string();
                }
                let mut s = String::from("{\n");
                let mut iter = borrow.iter().collect::<Vec<_>>();
                iter.sort_by_key(|a| a.0);
                for (i, (k, v)) in iter.iter().enumerate() {
                    s.push_str(&inner_spaces);
                    if k.contains(" ") || k.contains("-") {
                        s.push_str(&format!("\"{}\": ", k));
                    } else {
                        s.push_str(&format!("{}: ", k));
                    }
                    s.push_str(&v.to_string_pretty(indent + 2, false));
                    if i < borrow.len() - 1 {
                        s.push(',');
                    }
                    s.push('\n');
                }
                s.push_str(&format!("{}}}", spaces));
                s
            }
            Objek::String(val) => {
                if is_root {
                    val.clone()
                } else {
                    format!("\"{}\"", val)
                }
            }
            _ => format!("{}", self),
        }
    }
}

impl PartialEq for Objek {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Objek::Angka(a), Objek::Angka(b)) => a == b,
            (Objek::String(a), Objek::String(b)) => a == b,
            (Objek::Boolean(a), Objek::Boolean(b)) => a == b,
            (Objek::Kosong, Objek::Kosong) => true,
            (Objek::Kembalikan(a), Objek::Kembalikan(b)) => a == b,
            (Objek::Pengecualian(a), Objek::Pengecualian(b)) => a == b,
            (Objek::Array(a), Objek::Array(b)) => *a.borrow() == *b.borrow(),
            (Objek::Kamus(a), Objek::Kamus(b)) => *a.borrow() == *b.borrow(),
            _ => false,
        }
    }
}

impl fmt::Debug for Objek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Objek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Objek::Angka(val) => write!(f, "{}", val),
            Objek::String(val) => write!(f, "{}", val),
            Objek::Boolean(val) => write!(f, "{}", if *val { "benar" } else { "salah" }),
            Objek::Kosong => write!(f, "kosong"),
            Objek::Kembalikan(val) => write!(f, "{}", val),
            Objek::Pengecualian(val) => write!(f, "Pengecualian: {}", val),
            Objek::Fungsi { .. } => write!(f, "[Fungsi kustom]"),
            Objek::FungsiBawaan(_) => write!(f, "[Fungsi bawaan]"),
            Objek::MetodeBawaan(_) => write!(f, "[Metode bawaan]"),
            Objek::Modul(_) => write!(f, "[Modul]"),
            Objek::Array(elemen) => {
                let items: Vec<String> = elemen.borrow().iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Objek::Kamus(pasangan) => {
                let mut items: Vec<String> = pasangan
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                items.sort(); // Sort to ensure consistent output format
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

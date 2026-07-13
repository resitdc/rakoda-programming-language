use crate::objek::Objek;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Lingkungan {
    store: HashMap<String, Objek>,
    outer: Option<Rc<RefCell<Lingkungan>>>,
}

impl Lingkungan {
    pub fn baru() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            outer: None,
        }))
    }

    pub fn baru_nested(outer: Rc<RefCell<Lingkungan>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            outer: Some(outer),
        }))
    }

    pub fn get(&self, nama: &str) -> Option<Objek> {
        match self.store.get(nama) {
            Some(obj) => Some(obj.clone()),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(nama),
                None => None,
            },
        }
    }

    pub fn set(&mut self, nama: String, nilai: Objek) {
        self.store.insert(nama, nilai);
    }
}

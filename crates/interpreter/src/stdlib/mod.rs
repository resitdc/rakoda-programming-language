pub mod core;
pub mod env;
pub mod file;
pub mod http;
pub mod json;
pub mod kripto;
pub mod list;
pub mod matematika;
pub mod string;
pub mod waktu;

use crate::lingkungan::Lingkungan;
use std::cell::RefCell;
use std::rc::Rc;

pub fn register_all(env: &Rc<RefCell<Lingkungan>>) {
    core::register(env);
    matematika::register(env);
    string::register(env);
    list::register(env);
    file::register(env);
    waktu::register(env);
    json::register(env);
    http::register(env);
    env::register(env);
    kripto::register(env);
}

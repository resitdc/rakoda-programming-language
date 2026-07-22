pub mod adapter;
pub mod ai;
pub mod cookie;
pub mod core;
pub mod db;
pub mod dev_dashboard;
pub mod env;
pub mod file;
pub mod http;
pub mod json;
pub mod kripto;
pub mod list;
pub mod log;
pub mod matematika;
pub mod session;
pub mod string;
pub mod tugas;
pub mod waktu;
pub mod web;

pub mod regex;

use crate::machine::VM;

pub fn register_all(vm: &mut VM) {
    core::register(vm);
    waktu::register(vm);
    matematika::register(vm);
    list::register(vm);
    json::register(vm);
    http::register(vm);
    env::register(vm);
    file::register(vm);
    web::register(vm);
    cookie::register(vm);
    session::register(vm);
    tugas::register(vm);
    string::register(vm);
    db::register(vm);
    kripto::register(vm);
    log::register(vm);
    ai::register(vm);
    regex::register(vm);
}

pub mod adapter;
#[cfg(feature = "enterprise")]
pub mod ai;
#[cfg(feature = "enterprise")]
pub mod cookie;
pub mod core;
#[cfg(feature = "enterprise")]
pub mod db;
#[cfg(feature = "enterprise")]
pub mod dev_dashboard;
#[cfg(feature = "enterprise")]
pub mod dokumen;
#[cfg(feature = "enterprise")]
pub mod env;
#[cfg(feature = "enterprise")]
pub mod ffi;
#[cfg(feature = "enterprise")]
pub mod file;
#[cfg(feature = "enterprise")]
pub mod html;
#[cfg(feature = "enterprise")]
pub mod http;
pub mod json;
pub mod kripto;
pub mod list;
pub mod log;
pub mod matematika;
#[cfg(feature = "enterprise")]
pub mod session;
pub mod string;
#[cfg(feature = "enterprise")]
pub mod tugas;
pub mod waktu;
#[cfg(feature = "enterprise")]
pub mod web;

#[cfg(feature = "enterprise")]
pub mod cache;
#[cfg(feature = "enterprise")]
pub mod email;
#[cfg(feature = "enterprise")]
pub mod jwt;
#[cfg(feature = "enterprise")]
pub mod ktp;
#[cfg(feature = "enterprise")]
pub mod ocr;
pub mod regex;
pub mod tensor;
pub mod statistik;
pub mod linalg;
pub mod optim;

use crate::machine::VM;

pub fn register_all(vm: &mut VM) {
    core::register(vm);
    waktu::register(vm);
    matematika::register(vm);
    list::register(vm);
    json::register(vm);
    tensor::register(vm);
    statistik::register(vm);
    linalg::register(vm);
    optim::bawaan_optim(vm);
    #[cfg(feature = "enterprise")]
    http::register(vm);
    #[cfg(feature = "enterprise")]
    env::register(vm);
    #[cfg(feature = "enterprise")]
    file::register(vm);
    #[cfg(feature = "enterprise")]
    web::register(vm);
    #[cfg(feature = "enterprise")]
    cookie::register(vm);
    #[cfg(feature = "enterprise")]
    session::register(vm);
    #[cfg(feature = "enterprise")]
    tugas::register(vm);
    string::register(vm);
    #[cfg(feature = "enterprise")]
    db::register(vm);
    kripto::register(vm);
    regex::register(vm);
    log::register(vm);
    #[cfg(feature = "enterprise")]
    ai::register(vm);
    #[cfg(feature = "enterprise")]
    dokumen::register(vm);
    #[cfg(feature = "enterprise")]
    email::register(vm);
    #[cfg(feature = "enterprise")]
    jwt::register(vm);
    #[cfg(feature = "enterprise")]
    cache::register(vm);
    #[cfg(feature = "enterprise")]
    html::register(vm);
    #[cfg(feature = "enterprise")]
    ffi::register(vm);
    #[cfg(feature = "enterprise")]
    ocr::register(vm);
    #[cfg(feature = "enterprise")]
    ktp::register(vm);
}

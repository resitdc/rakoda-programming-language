pub mod core;
pub mod waktu;
pub mod matematika;
pub mod list;
pub mod json;
pub mod http;
pub mod env;
pub mod file;

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
}

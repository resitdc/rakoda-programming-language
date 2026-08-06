use libffi::middle::*;
fn main() {
    let args = vec![Type::i32()];
    let ret = Type::i32();
    let cif = Cif::new(args, ret);
}

#[cfg(feature = "enterprise")]
pub const FUNGSI: &[(&str, &[&str], &str)] = &[
    (
        "muat_lib",
        &["path"],
        "Memuat dynamic library (.dll, .so, .dylib) untuk digunakan dengan FFI.",
    ),
    (
        "definisi",
        &["lib", "nama_fungsi", "tipe_argumen", "tipe_kembalian"],
        "Mendefinisikan fungsi native FFI untuk dipanggil dari Rakoda.",
    ),
];

#[cfg(not(feature = "enterprise"))]
pub const FUNGSI: &[(&str, &[&str], &str)] = &[
    (
        "muat_lib",
        &["path"],
        "Memuat dynamic library (Membutuhkan fitur enterprise).",
    ),
    (
        "definisi",
        &["lib", "nama_fungsi", "tipe_argumen", "tipe_kembalian"],
        "Mendefinisikan fungsi native FFI (Membutuhkan fitur enterprise).",
    ),
];

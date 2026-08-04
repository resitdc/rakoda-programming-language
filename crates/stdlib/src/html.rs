#[cfg(feature = "enterprise")]
pub const FUNGSI: &[(&str, &[&str], &str)] = &[(
    "parse",
    &["teks"],
    "Parsing teks HTML menjadi objek dokumen.",
)];

#[cfg(not(feature = "enterprise"))]
pub const FUNGSI: &[(&str, &[&str], &str)] = &[(
    "parse",
    &["teks"],
    "Parsing teks HTML menjadi objek dokumen (Membutuhkan fitur enterprise).",
)];

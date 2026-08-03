use crate::{DaftarFungsiRpl, NilaiRpl};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

struct CacheItem {
    nilai: NilaiRpl,
    kedaluwarsa: Option<Instant>,
}

lazy_static! {
    static ref GLOBAL_CACHE: RwLock<HashMap<String, CacheItem>> = RwLock::new(HashMap::new());
}

pub fn fungsi_cache() -> DaftarFungsiRpl {
    vec![
        ("simpan", simpan_impl),
        ("ambil", ambil_impl),
        ("hapus", hapus_impl),
        ("cek", cek_impl),
        ("bersihkan", bersihkan_impl),
    ]
}

fn simpan_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("cache.simpan membutuhkan minimal 2 argumen: kunci, nilai".to_string());
    }

    let kunci = match &args[0] {
        NilaiRpl::Teks(k) => k.clone(),
        _ => return Err("Kunci cache harus berupa teks".to_string()),
    };

    let nilai = args[1].clone();

    let mut kedaluwarsa = None;
    if args.len() >= 3 {
        if let NilaiRpl::Angka(detik) = args[2] {
            if detik > 0.0 {
                kedaluwarsa = Some(Instant::now() + Duration::from_secs_f64(detik));
            }
        } else {
            return Err("Durasi kadaluwarsa harus berupa angka (detik)".to_string());
        }
    }

    let mut map = GLOBAL_CACHE.write().map_err(|_| "Gagal mengunci cache")?;
    map.insert(kunci, CacheItem { nilai, kedaluwarsa });

    Ok(NilaiRpl::Kosong)
}

fn ambil_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("cache.ambil membutuhkan argumen kunci".to_string());
    }

    let kunci = match &args[0] {
        NilaiRpl::Teks(k) => k,
        _ => return Err("Kunci cache harus berupa teks".to_string()),
    };

    let sekarang = Instant::now();
    let mut kadaluwarsa = false;

    // Baca dengan read-lock dulu
    let hasil = {
        let map = GLOBAL_CACHE.read().map_err(|_| "Gagal membaca cache")?;
        if let Some(item) = map.get(kunci) {
            if let Some(waktu) = item.kedaluwarsa {
                if sekarang >= waktu {
                    kadaluwarsa = true;
                    None
                } else {
                    Some(item.nilai.clone())
                }
            } else {
                Some(item.nilai.clone())
            }
        } else {
            None
        }
    };

    if kadaluwarsa {
        // Hapus jika sudah kedaluwarsa
        let mut map = GLOBAL_CACHE.write().map_err(|_| "Gagal mengunci cache")?;
        map.remove(kunci);
    }

    Ok(hasil.unwrap_or(NilaiRpl::Kosong))
}

fn hapus_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("cache.hapus membutuhkan argumen kunci".to_string());
    }

    let kunci = match &args[0] {
        NilaiRpl::Teks(k) => k,
        _ => return Err("Kunci cache harus berupa teks".to_string()),
    };

    let mut map = GLOBAL_CACHE.write().map_err(|_| "Gagal mengunci cache")?;
    map.remove(kunci);

    Ok(NilaiRpl::Kosong)
}

fn cek_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("cache.cek membutuhkan argumen kunci".to_string());
    }

    let kunci = match &args[0] {
        NilaiRpl::Teks(k) => k,
        _ => return Err("Kunci cache harus berupa teks".to_string()),
    };

    let sekarang = Instant::now();
    let mut kadaluwarsa = false;

    let ada = {
        let map = GLOBAL_CACHE.read().map_err(|_| "Gagal membaca cache")?;
        if let Some(item) = map.get(kunci) {
            if let Some(waktu) = item.kedaluwarsa {
                if sekarang >= waktu {
                    kadaluwarsa = true;
                    false
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            false
        }
    };

    if kadaluwarsa {
        let mut map = GLOBAL_CACHE.write().map_err(|_| "Gagal mengunci cache")?;
        map.remove(kunci);
    }

    Ok(NilaiRpl::Boolean(ada))
}

fn bersihkan_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    let mut map = GLOBAL_CACHE.write().map_err(|_| "Gagal mengunci cache")?;
    map.clear();

    Ok(NilaiRpl::Kosong)
}

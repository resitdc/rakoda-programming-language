use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;

/// Fungsi-fungsi list murni. List direpresentasikan sebagai Vec<NilaiRpl>.
/// Operasi menghasilkan list baru (immutable), kecuali engine mendukung mutasi.
pub fn fungsi_list() -> DaftarFungsiRpl {
    vec![
        ("tambah", tambah_impl),
        ("hapus", hapus_impl),
        ("panjang", panjang_impl),
        ("ambil", ambil_impl),
    ]
}

fn tambah_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("list.tambah membutuhkan 2 argumen: list, nilai".to_string());
    }
    match &args[0] {
        NilaiRpl::Daftar(arr) => {
            let mut new_arr = arr.clone();
            new_arr.push(args[1].clone());
            Ok(NilaiRpl::Daftar(new_arr))
        }
        _ => Err("list.tambah: argumen pertama harus daftar".to_string()),
    }
}

fn hapus_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("list.hapus membutuhkan 2 argumen: daftar, indeks".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Daftar(arr), NilaiRpl::Angka(idx)) => {
            let index = *idx as usize;
            if index >= arr.len() {
                return Err(format!(
                    "list.hapus: indeks {} di luar jangkauan (panjang: {})",
                    index,
                    arr.len()
                ));
            }
            let mut new_arr = arr.clone();
            new_arr.remove(index);
            Ok(NilaiRpl::Daftar(new_arr))
        }
        _ => Err("list.hapus: argumen harus daftar, angka".to_string()),
    }
}

fn panjang_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("list.panjang membutuhkan 1 argumen: daftar".to_string());
    }
    match &args[0] {
        NilaiRpl::Daftar(arr) => Ok(NilaiRpl::Angka(arr.len() as f64)),
        _ => Err("list.panjang hanya menerima daftar".to_string()),
    }
}

fn ambil_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("list.ambil membutuhkan 2 argumen: daftar, indeks".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Daftar(arr), NilaiRpl::Angka(idx)) => {
            let index = *idx as usize;
            arr.get(index)
                .cloned()
                .ok_or_else(|| format!("list.ambil: indeks {} di luar jangkauan", index))
        }
        _ => Err("list.ambil: argumen harus daftar, angka".to_string()),
    }
}

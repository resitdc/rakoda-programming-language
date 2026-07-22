use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;

/// Fungsi-fungsi waktu murni. Tidak bergantung pada sumber waktu sistem.
/// Engine akan menyediakan DateTime<Local> atau f64 UNIX timestamp saat memanggil.
/// Untuk simplicity saat ini, gunakan `chrono::Local` langsung (bisa di-override engine nanti).
pub fn fungsi_waktu() -> DaftarFungsiRpl {
    vec![
        ("sekarang", sekarang_impl),
        ("tahun", tahun_impl),
        ("bulan", bulan_impl),
        ("tanggal", tanggal_impl),
        ("jam", jam_impl),
        ("menit", menit_impl),
        ("detik", detik_impl),
        ("format", format_impl),
        ("relatif", relatif_impl),
    ]
}

use chrono::{Datelike, Local, Timelike};

fn sekarang_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().timestamp() as f64))
}

fn tahun_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().year() as f64))
}

fn bulan_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().month() as f64))
}

fn tanggal_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().day() as f64))
}

fn jam_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().hour() as f64))
}

fn menit_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().minute() as f64))
}

fn detik_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Angka(Local::now().second() as f64))
}

fn format_impl(_args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    Ok(NilaiRpl::Teks(
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    ))
}

fn relatif_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("waktu.relatif membutuhkan 1 argumen: timestamp".to_string());
    }
    match &args[0] {
        NilaiRpl::Angka(val) => {
            let timestamp_sekarang = Local::now().timestamp();
            let timestamp_arg = *val as i64;
            let selisih = timestamp_sekarang - timestamp_arg;
            
            if selisih < 0 {
                return Ok(NilaiRpl::Teks("Di masa depan".to_string()));
            }
            
            let hasil = if selisih < 60 {
                "Baru saja".to_string()
            } else if selisih < 3600 {
                let menit = selisih / 60;
                format!("{} menit yang lalu", menit)
            } else if selisih < 86400 {
                let jam = selisih / 3600;
                format!("{} jam yang lalu", jam)
            } else if selisih < 172800 { // 48 jam
                "Kemarin".to_string()
            } else if selisih < 2592000 { // 30 hari
                let hari = selisih / 86400;
                format!("{} hari yang lalu", hari)
            } else if selisih < 31536000 { // 365 hari
                let bulan = selisih / 2592000;
                format!("{} bulan yang lalu", bulan)
            } else {
                let tahun = selisih / 31536000;
                format!("{} tahun yang lalu", tahun)
            };
            
            Ok(NilaiRpl::Teks(hasil))
        }
        _ => Err("waktu.relatif hanya menerima angka timestamp".to_string()),
    }
}

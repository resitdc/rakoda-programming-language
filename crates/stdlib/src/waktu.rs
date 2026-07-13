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

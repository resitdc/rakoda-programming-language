use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;

/// Fungsi-fungsi string murni untuk manipulasi teks.
pub fn fungsi_string() -> DaftarFungsiRpl {
    vec![
        ("panjang", panjang_impl),
        ("besar", besar_impl),
        ("kecil", kecil_impl),
        ("potong", potong_impl),
        ("ganti", ganti_impl),
        ("pecah", pecah_impl),
        ("kandung", kandung_impl),
        ("bersih", bersih_impl),
        ("trim", bersih_impl),
        ("terbilang", terbilang_impl),
        ("format_rupiah", format_rupiah_impl),
        ("alay", alay_impl),
        ("sensor", sensor_impl),
    ]
}

fn panjang_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.panjang membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Angka(s.len() as f64)),
        _ => Err("string.panjang hanya menerima teks".to_string()),
    }
}

fn besar_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.besar membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Teks(s.to_uppercase())),
        _ => Err("string.besar hanya menerima teks".to_string()),
    }
}

fn kecil_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.kecil membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Teks(s.to_lowercase())),
        _ => Err("string.kecil hanya menerima teks".to_string()),
    }
}

fn potong_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 3 {
        return Err("string.potong membutuhkan 3 argumen: teks, mulai, selesai".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (NilaiRpl::Teks(s), NilaiRpl::Angka(mulai), NilaiRpl::Angka(selesai)) => {
            let mulai = *mulai as usize;
            let selesai = *selesai as usize;
            if mulai > s.len() || selesai > s.len() || mulai > selesai {
                return Err("string.potong: indeks di luar jangkauan".to_string());
            }
            Ok(NilaiRpl::Teks(s[mulai..selesai].to_string()))
        }
        _ => Err("string.potong: argumen harus teks, angka, angka".to_string()),
    }
}

fn ganti_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 3 {
        return Err("string.ganti membutuhkan 3 argumen: teks, cari, ganti".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (NilaiRpl::Teks(s), NilaiRpl::Teks(cari), NilaiRpl::Teks(ganti)) => {
            Ok(NilaiRpl::Teks(s.replace(cari, ganti)))
        }
        _ => Err("string.ganti: argumen harus teks, teks, teks".to_string()),
    }
}

fn pecah_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("string.pecah membutuhkan 2 argumen: teks, pemisah".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Teks(s), NilaiRpl::Teks(pemisah)) => {
            let split: Vec<NilaiRpl> = s
                .split(pemisah)
                .map(|p| NilaiRpl::Teks(p.to_string()))
                .collect();
            Ok(NilaiRpl::Daftar(split))
        }
        _ => Err("string.pecah: argumen harus teks dan teks".to_string()),
    }
}

fn kandung_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.len() < 2 {
        return Err("string.kandung membutuhkan 2 argumen: teks, kata".to_string());
    }
    match (&args[0], &args[1]) {
        (NilaiRpl::Teks(s), NilaiRpl::Teks(kata)) => {
            Ok(NilaiRpl::Boolean(s.contains(kata)))
        }
        _ => Err("string.kandung: argumen harus teks dan teks".to_string()),
    }
}

fn bersih_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.bersih membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => Ok(NilaiRpl::Teks(s.trim().to_string())),
        _ => Err("string.bersih hanya menerima teks".to_string()),
    }
}

fn terbilang_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.terbilang membutuhkan 1 argumen: angka".to_string());
    }
    match &args[0] {
        NilaiRpl::Angka(val) => {
            let num = *val as i64;
            Ok(NilaiRpl::Teks(angka_ke_terbilang(num)))
        }
        _ => Err("string.terbilang hanya menerima angka".to_string()),
    }
}

fn angka_ke_terbilang(n: i64) -> String {
    if n < 0 {
        return format!("minus {}", angka_ke_terbilang(-n));
    }
    if n == 0 {
        return "nol".to_string();
    }
    
    let satuan = ["", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh", "delapan", "sembilan", "sepuluh", "sebelas"];
    
    if n < 12 {
        return satuan[n as usize].to_string();
    } else if n < 20 {
        return format!("{} belas", satuan[(n - 10) as usize]);
    } else if n < 100 {
        let puluhan = n / 10;
        let sisa = n % 10;
        let sisa_str = if sisa > 0 { format!(" {}", satuan[sisa as usize]) } else { "".to_string() };
        return format!("{} puluh{}", satuan[puluhan as usize], sisa_str);
    } else if n < 200 {
        let sisa = n - 100;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("seratus{}", sisa_str);
    } else if n < 1000 {
        let ratusan = n / 100;
        let sisa = n % 100;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("{} ratus{}", satuan[ratusan as usize], sisa_str);
    } else if n < 2000 {
        let sisa = n - 1000;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("seribu{}", sisa_str);
    } else if n < 1_000_000 {
        let ribuan = n / 1000;
        let sisa = n % 1000;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("{} ribu{}", angka_ke_terbilang(ribuan), sisa_str);
    } else if n < 1_000_000_000 {
        let jutaan = n / 1_000_000;
        let sisa = n % 1_000_000;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("{} juta{}", angka_ke_terbilang(jutaan), sisa_str);
    } else if n < 1_000_000_000_000 {
        let miliaran = n / 1_000_000_000;
        let sisa = n % 1_000_000_000;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("{} miliar{}", angka_ke_terbilang(miliaran), sisa_str);
    } else {
        let triliunan = n / 1_000_000_000_000;
        let sisa = n % 1_000_000_000_000;
        let sisa_str = if sisa > 0 { format!(" {}", angka_ke_terbilang(sisa)) } else { "".to_string() };
        return format!("{} triliun{}", angka_ke_terbilang(triliunan), sisa_str);
    }
}

fn format_rupiah_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.format_rupiah membutuhkan 1 argumen: angka".to_string());
    }
    match &args[0] {
        NilaiRpl::Angka(val) => {
            let val = *val;
            let int_part = val.trunc() as i64;
            let s = int_part.to_string();
            
            // Tambahkan titik setiap 3 digit
            let mut result = String::new();
            let mut count = 0;
            for c in s.chars().rev() {
                if count > 0 && count % 3 == 0 {
                    result.insert(0, '.');
                }
                result.insert(0, c);
                count += 1;
            }
            Ok(NilaiRpl::Teks(format!("Rp {}", result)))
        }
        _ => Err("string.format_rupiah hanya menerima angka".to_string()),
    }
}

fn alay_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.alay membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => {
            let s_alay = s
                .replace("a", "4").replace("A", "4")
                .replace("i", "1").replace("I", "1")
                .replace("e", "3").replace("E", "3")
                .replace("s", "5").replace("S", "5")
                .replace("g", "9").replace("G", "9")
                .replace("o", "0").replace("O", "0");
            Ok(NilaiRpl::Teks(s_alay))
        }
        _ => Err("string.alay hanya menerima teks".to_string()),
    }
}

fn sensor_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("string.sensor membutuhkan 1 argumen: teks".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => {
            // Sensor canggih mendeteksi kata-kata kasar dengan substitusi karakter (leetspeak)
            use regex::RegexBuilder;
            
            // Kata kasar yang umum dengan huruf alternatif (contoh)
            // k -> k, c, q, dll. Tapi kita pakai pola umum untuk vokal leet.
            // A -> [aA4@], I -> [iI1!l|], E -> [eE3], O -> [oO0], S -> [sS5$]
            let pattern = r"(?i)\b(
                a[nN][jJ][iI1!l|][nN][gG9]|
                b[aA4@][nN][gG9]?[sS5$][aA4@][tT]|
                t[aA4@]?[iI1!l|]|
                [bB][oO0][dD][oO0][hH]|
                [gG9][oO0][bB][lL|1!][oO0][kK]|
                [tT][oO0][lL|1!][oO0][lL|1!]|
                [jJ][aA4@][nN][cC][uU][kK]
            )\b";
            
            // Hilangkan spasi dan newline multiline regex (kita hapus spasi/newline manual untuk membuat string regex-nya)
            let raw_pattern = pattern.replace(&['\n', ' ', '\t'][..], "");
            
            if let Ok(re) = RegexBuilder::new(&raw_pattern)
                .case_insensitive(true)
                .build() 
            {
                // Ganti kata-kata kasar dengan panjang '*' yang sesuai dengan panjang aslinya
                let hasil = re.replace_all(s, |caps: &regex::Captures| {
                    let mat = caps.get(0).unwrap().as_str();
                    "*".repeat(mat.len())
                });
                return Ok(NilaiRpl::Teks(hasil.to_string()));
            }
            Ok(NilaiRpl::Teks(s.clone()))
        }
        _ => Err("string.sensor hanya menerima teks".to_string()),
    }
}

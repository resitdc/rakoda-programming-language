use crate::value::Value;
use crate::value::VmContext;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use ocrs::{ImageSource, DimOrder};
use rten_tensor::prelude::*;

fn normalisasi_angka(text: &str) -> String {
    let mut hasil = String::new();
    for c in text.chars() {
        match c {
            'O' | 'o' | 'D' | 'Q' => hasil.push('0'),
            'I' | 'l' | 'L' | '|' | '!' | 'i' => hasil.push('1'),
            'Z' | 'z' => hasil.push('2'),
            'S' | 's' => hasil.push('5'),
            'b' | 'G' | 'g' => hasil.push('6'),
            'T' | 't' | '?' => hasil.push('7'),
            'B' => hasil.push('8'),
            'e' => hasil.push('8'),
            'q' => hasil.push('9'),
            _ if c.is_ascii_digit() => hasil.push(c),
            _ => {},
        }
    }
    hasil
}

fn bersih_teks(text: &str) -> String {
    let upper = text.to_uppercase();
    // Hilangkan karakter aneh
    upper.chars().filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '.' || *c == ',' || *c == '/').collect()
}

pub fn register(vm: &mut crate::VM) {
    let mut map = HashMap::new();

    let baca_func = crate::value::FungsiBawaanVM {
        nama: "ktp.baca".to_string(),
        func: Arc::new(|ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
            if args.len() != 1 {
                return Err("Fungsi 'baca' membutuhkan 1 argumen path_gambar".to_string());
            }

            let path_str = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen path_gambar harus berupa string".to_string()),
            };

            let path = Path::new(&path_str);
            if !path.exists() {
                return Err(format!("File gambar tidak ditemukan: {}", path_str));
            }

            let engine = super::ocr::inisialisasi_engine()?;

            let dyn_image = match image::open(&path_str) {
                Ok(i) => i,
                Err(e) => return Err(format!("Gagal membaca gambar: {}", e)),
            };

            // Pra-pemrosesan di-skip karena seringkali menurunkan kualitas pembacaan model OCRS
            // jika gambar sudah lumayan jelas.

            let img = match rten_imageio::image_to_tensor(dyn_image) {
                Ok(i) => i,
                Err(e) => return Err(format!("Gagal konversi ke tensor: {}", e)),
            };

            let img_source = match ImageSource::from_tensor(img.view(), DimOrder::Chw) {
                Ok(src) => src,
                Err(e) => return Err(format!("Gagal membuat sumber gambar: {:?}", e)),
            };

            let ocr_input = match engine.prepare_input(img_source) {
                Ok(i) => i,
                Err(e) => return Err(format!("Gagal memproses gambar untuk OCR: {}", e)),
            };

            let word_rects = match engine.detect_words(&ocr_input) {
                Ok(w) => w,
                Err(e) => return Err(format!("Gagal mendeteksi teks: {}", e)),
            };

            let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

            let lines = match engine.recognize_text(&ocr_input, &line_rects) {
                Ok(l) => l,
                Err(e) => return Err(format!("Gagal mengenali teks: {}", e)),
            };

            let mut hasil_baris = Vec::new();
            for line in lines.iter().flatten() {
                let text = line.to_string();
                if text.trim().len() > 0 {
                    hasil_baris.push(text);
                }
            }

            let mut provinsi = String::new();
            let mut kota_kabupaten = String::new();
            let mut nik = String::new();
            let mut nama = String::new();
            let mut tempat_lahir = String::new();
            let mut tanggal_lahir = String::new();
            let mut jenis_kelamin = String::new();
            let mut golongan_darah = String::new();
            let mut alamat = String::new();
            let mut rt = String::new();
            let mut rw = String::new();
            let mut kelurahan_desa = String::new();
            let mut kecamatan = String::new();
            let mut agama = String::new();
            let mut status_perkawinan = String::new();
            let mut pekerjaan = String::new();
            let mut kewarganegaraan = String::new();
            let mut berlaku_hingga = String::new();

            for (i, baris) in hasil_baris.iter().enumerate() {
                let b_upper = bersih_teks(baris);
                
                if b_upper.contains("PROVINSI") {
                    provinsi = b_upper.replace("PROVINSI", "").trim().to_string();
                } else if b_upper.contains("KOTA") || b_upper.contains("KABUPATEN") {
                    kota_kabupaten = b_upper.trim().to_string();
                } else if b_upper.contains("NIK") || normalisasi_angka(&b_upper).len() >= 16 {
                    if nik.is_empty() {
                        let n_angka = normalisasi_angka(&b_upper);
                        if n_angka.len() >= 16 {
                            nik = n_angka[0..16].to_string();
                            if i + 1 < hasil_baris.len() && nama.is_empty() {
                                let nama_baris = bersih_teks(&hasil_baris[i + 1]);
                                let nama_bersih = nama_baris.replace("NAMA", "").replace(":", "").trim().to_string();
                                if nama_bersih.len() > 0 && !nama_bersih.contains("TEMPAT") {
                                    nama = nama_bersih;
                                }
                            }
                        }
                    }
                }

                if b_upper.contains("NAMA") && nama.is_empty() {
                    nama = b_upper.split("NAMA").last().unwrap_or("").replace(":", "").trim().to_string();
                }

                if b_upper.contains("LAHIR") || b_upper.contains("LHR") {
                    let text_lahir = b_upper.split("LAHIR").last().unwrap_or("").replace(":", "").trim().to_string();
                    let parts: Vec<&str> = text_lahir.split(',').collect();
                    if parts.len() >= 2 {
                        tempat_lahir = parts[0].trim().to_string();
                        let angka_tgl = normalisasi_angka(parts[1]);
                        if angka_tgl.len() >= 8 {
                            tanggal_lahir = format!("{}-{}-{}", &angka_tgl[0..2], &angka_tgl[2..4], &angka_tgl[4..8]);
                        } else {
                            tanggal_lahir = parts[1].trim().to_string();
                        }
                    } else {
                        let angka_tgl = normalisasi_angka(&text_lahir);
                        if angka_tgl.len() >= 8 {
                            tanggal_lahir = format!("{}-{}-{}", &angka_tgl[0..2], &angka_tgl[2..4], &angka_tgl[4..8]);
                            let tempat = text_lahir.replace(parts.last().unwrap_or(&""), "").trim().to_string();
                            tempat_lahir = tempat;
                        }
                    }
                }

                if b_upper.contains("LAKI") || b_upper.contains("PEREMPUAN") || b_upper.contains("KELAMIN") {
                    if b_upper.contains("PEREMPUAN") {
                        jenis_kelamin = "PEREMPUAN".to_string();
                    } else {
                        jenis_kelamin = "LAKI-LAKI".to_string();
                    }
                }

                if b_upper.contains("GOL") || b_upper.contains("DARAH") {
                    let p = b_upper.split_whitespace().last().unwrap_or("");
                    if p == "A" || p == "B" || p == "AB" || p == "O" || p == "0" {
                        golongan_darah = p.replace("0", "O");
                    } else {
                        golongan_darah = "-".to_string();
                    }
                }

                if b_upper.contains("ALAMAT") {
                    alamat = b_upper.split("ALAMAT").last().unwrap_or("").replace(":", "").trim().to_string();
                }
                if b_upper.contains("RT/RW") || b_upper.contains("RTRW") {
                    let text_rtrw = b_upper.split("RW").last().unwrap_or(&b_upper);
                    let val = normalisasi_angka(text_rtrw);
                    if val.len() >= 6 {
                        rt = val[0..3].to_string();
                        rw = val[3..6].to_string();
                    } else if val.len() >= 3 {
                        rt = val[0..3].to_string();
                    }
                } else if b_upper.contains("RT") && rt.is_empty() {
                     let text_rt = b_upper.split("RT").last().unwrap_or(&b_upper);
                     let val = normalisasi_angka(text_rt);
                     if val.len() >= 3 {
                        rt = val[0..3].to_string();
                     }
                }

                if b_upper.contains("KEL/DESA") || b_upper.contains("KELURAHAN") {
                    kelurahan_desa = b_upper.split("DESA").last().unwrap_or(&b_upper).replace(":", "").trim().to_string();
                }
                if b_upper.contains("KECAMATAN") || b_upper.contains("KEC") {
                    kecamatan = b_upper.split("KECAMATAN").last().unwrap_or(&b_upper).replace(":", "").trim().to_string();
                }
                
                if b_upper.contains("AGAMA") {
                    let agama_str = b_upper.split("AGAMA").last().unwrap_or("").replace(":", "").trim().to_string();
                    if agama_str.contains("ISAM") || agama_str.contains("ISLAM") { agama = "ISLAM".to_string(); }
                    else if agama_str.contains("KRIS") { agama = "KRISTEN".to_string(); }
                    else if agama_str.contains("KAT") { agama = "KATOLIK".to_string(); }
                    else if agama_str.contains("HIN") { agama = "HINDU".to_string(); }
                    else if agama_str.contains("BUD") { agama = "BUDDHA".to_string(); }
                    else if agama_str.contains("KHON") { agama = "KHONGHUCU".to_string(); }
                }

                if b_upper.contains("KAWIN") || b_upper.contains("STATUS") {
                    if b_upper.contains("BELUM") { status_perkawinan = "BELUM KAWIN".to_string(); }
                    else if b_upper.contains("CERAI MATI") { status_perkawinan = "CERAI MATI".to_string(); }
                    else if b_upper.contains("CERAI") { status_perkawinan = "CERAI HIDUP".to_string(); }
                    else { status_perkawinan = "KAWIN".to_string(); }
                }

                if b_upper.contains("PEKERJAAN") {
                    pekerjaan = b_upper.split("PEKERJAAN").last().unwrap_or("").replace(":", "").trim().to_string();
                }

                if b_upper.contains("KEWARGANEGARAAN") || b_upper.contains("WNI") || b_upper.contains("WNA") {
                    if b_upper.contains("WNA") || b_upper.ends_with("A") {
                        kewarganegaraan = "WNA".to_string();
                    } else {
                        kewarganegaraan = "WNI".to_string();
                    }
                }

                if b_upper.contains("BERLAKU") {
                    berlaku_hingga = "SEUMUR HIDUP".to_string();
                }
            }

            macro_rules! make_str {
                ($s:expr) => {
                    Value::String(ctx.get_heap_mut().alloc(crate::heap::HeapData::String($s)))
                }
            }

            let mut dict_alamat = HashMap::new();
            dict_alamat.insert("alamat".to_string(), make_str!(alamat));
            dict_alamat.insert("rt".to_string(), make_str!(rt));
            dict_alamat.insert("rw".to_string(), make_str!(rw));
            dict_alamat.insert("kelurahan_desa".to_string(), make_str!(kelurahan_desa));
            dict_alamat.insert("kecamatan".to_string(), make_str!(kecamatan));
            let dict_alamat_ptr = ctx.get_heap_mut().alloc(crate::heap::HeapData::Kamus(dict_alamat));

            let mut dict = HashMap::new();
            dict.insert("provinsi".to_string(), make_str!(provinsi));
            dict.insert("kota_kabupaten".to_string(), make_str!(kota_kabupaten));
            dict.insert("nik".to_string(), make_str!(nik));
            dict.insert("nama".to_string(), make_str!(nama));
            dict.insert("tempat_lahir".to_string(), make_str!(tempat_lahir));
            dict.insert("tanggal_lahir".to_string(), make_str!(tanggal_lahir));
            dict.insert("jenis_kelamin".to_string(), make_str!(jenis_kelamin));
            dict.insert("golongan_darah".to_string(), make_str!(golongan_darah));
            dict.insert("alamat_lengkap".to_string(), Value::Kamus(dict_alamat_ptr));
            dict.insert("agama".to_string(), make_str!(agama));
            dict.insert("status_perkawinan".to_string(), make_str!(status_perkawinan));
            dict.insert("pekerjaan".to_string(), make_str!(pekerjaan));
            dict.insert("kewarganegaraan".to_string(), make_str!(kewarganegaraan));
            dict.insert("berlaku_hingga".to_string(), make_str!(berlaku_hingga));

            let dict_ptr = ctx.get_heap_mut().alloc(crate::heap::HeapData::Kamus(dict));
            Ok(Value::Kamus(dict_ptr))
        }),
    };

    let baca_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(baca_func));
    map.insert("baca".to_string(), Value::FungsiBawaan(baca_idx));

    let ptr = vm.heap.alloc(crate::heap::HeapData::Kamus(map));
    vm.environments[0].insert("ktp".to_string(), Value::Kamus(ptr));
}

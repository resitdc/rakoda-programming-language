use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;
use std::collections::HashMap;
use std::path::Path;

pub fn fungsi_dokumen() -> DaftarFungsiRpl {
    vec![("baca", baca_impl)]
}

fn baca_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("dokumen.baca membutuhkan 1 argumen: path_file".to_string());
    }

    let path_str = match &args[0] {
        NilaiRpl::Teks(p) => p,
        _ => return Err("argumen dokumen.baca harus berupa teks (path)".to_string()),
    };

    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("File tidak ditemukan: {}", path_str));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "pdf" => baca_pdf(path),
        "docx" => baca_docx(path),
        "xlsx" | "xls" => baca_excel(path),
        _ => Err(format!(
            "Format dokumen '{}' belum didukung. Gunakan pdf, docx, xlsx, atau xls.",
            ext
        )),
    }
}

fn baca_pdf(path: &Path) -> Result<NilaiRpl, String> {
    match pdf_extract::extract_text(path) {
        Ok(teks) => Ok(NilaiRpl::Teks(teks)),
        Err(e) => Err(format!("Gagal membaca PDF: {:?}", e)),
    }
}

fn baca_docx(path: &Path) -> Result<NilaiRpl, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Gagal membuka file: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Gagal membuka archive DOCX: {}", e))?;

    let mut document_xml = archive
        .by_name("word/document.xml")
        .map_err(|e| format!("Gagal menemukan word/document.xml: {}", e))?;
    let mut isi = String::new();
    std::io::Read::read_to_string(&mut document_xml, &mut isi)
        .map_err(|e| format!("Gagal membaca XML DOCX: {}", e))?;

    let mut reader = quick_xml::Reader::from_str(&isi);
    let config = reader.config_mut();
    config.trim_text(true);
    let mut teks = String::new();
    let mut in_text_node = false;

    use quick_xml::events::Event;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text_node = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text_node = false;
            }
            Ok(Event::Text(e)) if in_text_node => {
                let t = String::from_utf8_lossy(e.as_ref());
                teks.push_str(&t);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                teks.push('\n');
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Gagal mem-parsing XML DOCX: {}", e)),
            _ => (),
        }
    }

    Ok(NilaiRpl::Teks(teks))
}

fn baca_excel(path: &Path) -> Result<NilaiRpl, String> {
    use calamine::{Data, Reader, open_workbook_auto};

    let mut workbook =
        open_workbook_auto(path).map_err(|e| format!("Gagal membuka Excel: {}", e))?;

    let mut sheets = HashMap::new();

    let sheet_names = workbook.sheet_names().to_owned();
    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut baris_daftar = Vec::new();

            for row in range.rows() {
                let mut sel_daftar = Vec::new();
                for cell in row {
                    let nilai_sel = match cell {
                        Data::String(s) => NilaiRpl::Teks(s.to_string()),
                        Data::Float(f) => NilaiRpl::Angka(*f),
                        Data::Int(i) => NilaiRpl::Angka(*i as f64),
                        Data::Bool(b) => NilaiRpl::Boolean(*b),
                        Data::Empty | Data::Error(_) => NilaiRpl::Kosong,
                        Data::DateTime(d) => NilaiRpl::Angka(d.as_f64()),
                        Data::DateTimeIso(s) => NilaiRpl::Teks(s.to_string()),
                        Data::DurationIso(s) => NilaiRpl::Teks(s.to_string()),
                    };
                    sel_daftar.push(nilai_sel);
                }
                baris_daftar.push(NilaiRpl::Daftar(sel_daftar));
            }

            sheets.insert(sheet_name, NilaiRpl::Daftar(baris_daftar));
        }
    }

    Ok(NilaiRpl::Kamus(sheets))
}

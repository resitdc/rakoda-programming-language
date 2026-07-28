use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

fn extract_text_from_html(html: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut clean = String::new();
    let mut in_tag = false;

    let replaced = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("</p>", "\n")
        .replace("</h1>", "\n")
        .replace("</h2>", "\n")
        .replace("</h3>", "\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n");

    for c in replaced.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            clean.push(c);
        }
    }

    for line in clean.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            paragraphs.push(trimmed.to_string());
        }
    }

    paragraphs
}

pub fn register(vm: &mut VM) {
    let mut dokumen_module = HashMap::new();

    let buat_func = FungsiBawaanVM {
        nama: "buat".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 3 {
                    return Err(
                        "Fungsi 'dokumen.buat' membutuhkan 3 argumen: sumber, nama_file, jenis"
                            .to_string(),
                    );
                }

                let jenis = match &args[2] {
                    Value::String(idx) => {
                        ctx.get_heap_mut().get_string(*idx).clone().to_lowercase()
                    }
                    _ => {
                        return Err(
                            "Argumen 'jenis' harus berupa teks ('docx', 'pdf', atau 'excel')"
                                .to_string(),
                        );
                    }
                };

                let nama_file = match &args[1] {
                    Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                    _ => return Err("Argumen 'nama_file' harus berupa teks".to_string()),
                };

                let ext = if jenis == "excel" { "xlsx" } else { &jenis };

                let mut final_filename = if nama_file.ends_with(&format!(".{}", ext)) {
                    nama_file.clone()
                } else {
                    format!("{}.{}", nama_file, ext)
                };

                // Pastikan file disimpan di direktori script (project_root), bukan di CWD eksekusi
                if let Some(root) = &ctx.get_heap_mut().project_root {
                    let path = std::path::Path::new(&final_filename);
                    if path.is_relative() {
                        final_filename = root.join(path).to_string_lossy().into_owned();
                    }
                }

                if jenis == "excel" {
                    return crate::stdlib::dokumen::build_excel(ctx, &args[0], &final_filename);
                }

                let sumber = match &args[0] {
                    Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                    _ => {
                        return Err(format!(
                            "Argumen 'sumber' harus berupa teks untuk tipe {}",
                            jenis
                        ));
                    }
                };

                let html_content =
                    if sumber.ends_with(".html") && std::path::Path::new(&sumber).exists() {
                        std::fs::read_to_string(&sumber).unwrap_or(sumber.clone())
                    } else {
                        sumber.clone()
                    };

                let paragraphs = extract_text_from_html(&html_content);

                if jenis == "docx" {
                    use docx_rs::*;
                    let mut doc = Docx::new();
                    for p in paragraphs {
                        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(p)));
                    }
                    let file = std::fs::File::create(&final_filename).map_err(|e| e.to_string())?;
                    doc.build().pack(file).map_err(|e| e.to_string())?;
                } else if jenis == "pdf" {
                    use turbo_html2pdf_core::{
                        CompileOptions, Diagnostics, EmitOptions, FontRegistry, NoImages,
                        RenderInputs, build_cascade, compile, emit_pdf, render_pages,
                        style::TokenSet,
                    };

                    let mut final_html = html_content.clone();
                    // Jika sumber bukan dokumen HTML penuh, kita asumsikan teks biasa / snippet
                    // Ubah \n menjadi <br/> agar baris baru dirender dengan benar di PDF
                    if !final_html.to_lowercase().contains("<html")
                        && !final_html.to_lowercase().contains("<body")
                    {
                        final_html = final_html.replace("\n", "<br/>");
                        // Tambahkan styling dasar agar teks lebih rapi jika tidak ada style
                        final_html = format!(
                            "<div style=\"font-family: sans-serif; font-size: 14px; line-height: 1.5;\">{}</div>",
                            final_html
                        );
                    }

                    let (program, _diags) = compile(&final_html, &CompileOptions::default())
                        .map_err(|e| format!("Gagal kompilasi HTML: {:?}", e.code))?;

                    let data = serde_json::Value::Null;

                    let mut author_css = String::new();
                    if let (Some(start), Some(end)) =
                        (final_html.find("<style>"), final_html.find("</style>"))
                        && start < end
                    {
                        author_css = final_html[start + 7..end].to_string();
                    }

                    let cascade = build_cascade(&author_css, "", TokenSet::default());
                    let fonts = FontRegistry::new();
                    let rules = turbo_html2pdf_core::style::parse_stylesheet(&author_css).at_rules;

                    let inputs = RenderInputs {
                        program: &program,
                        data: &data,
                        cascade: &cascade,
                        at_rules: &rules,
                        fonts: &fonts,
                        images: &NoImages,
                        now: Some(0),
                    };

                    let mut diags = Diagnostics::default();
                    let pages = render_pages(&inputs, &mut diags)
                        .map_err(|e| format!("Gagal render PDF: {:?}", e.code))?;

                    let pdf_bytes = emit_pdf(&pages, &EmitOptions::default());

                    std::fs::write(&final_filename, pdf_bytes).map_err(|e| e.to_string())?;
                } else {
                    return Err(
                        "Jenis laporan tidak didukung. Gunakan 'docx' atau 'pdf'.".to_string()
                    );
                }

                Ok(Value::Kosong)
            },
        ),
    };

    let buat_idx = vm.heap.alloc(HeapData::FungsiBawaan(buat_func));
    dokumen_module.insert("buat".to_string(), Value::FungsiBawaan(buat_idx));

    let dokumen_idx = vm.heap.alloc(HeapData::Kamus(dokumen_module));
    vm.environments[0].insert("dokumen".to_string(), Value::Kamus(dokumen_idx));
}

fn build_excel(
    ctx: &mut dyn VmContext,
    sumber: &Value,
    final_filename: &str,
) -> Result<Value, String> {
    use rust_xlsxwriter::{Format, Workbook};

    let mut workbook = Workbook::new();

    let process_sheet = |workbook: &mut Workbook,
                         sheet_data: &HashMap<String, Value>,
                         ctx: &mut dyn VmContext|
     -> Result<(), String> {
        // Resolve sheet name
        let mut sheet_name = String::new();
        for key in &["lembar", "judul", "title"] {
            if let Some(Value::String(idx)) = sheet_data.get(*key) {
                sheet_name = ctx.get_heap_mut().get_string(*idx).clone();
                break;
            }
        }

        let worksheet = if sheet_name.is_empty() {
            workbook.add_worksheet()
        } else {
            workbook
                .add_worksheet()
                .set_name(&sheet_name)
                .map_err(|e| e.to_string())?
        };

        // Process cells
        let mut cells_map = None;
        for key in &["sel", "cell"] {
            if let Some(Value::Kamus(idx)) = sheet_data.get(*key) {
                cells_map = Some(ctx.get_heap_mut().get_kamus(*idx).clone());
                break;
            }
        }

        let parse_cell_ref = |cell_ref: &str| -> Option<(u32, u16)> {
            let mut col_str = String::new();
            let mut row_str = String::new();
            for c in cell_ref.chars() {
                if c.is_alphabetic() {
                    col_str.push(c);
                } else if c.is_numeric() {
                    row_str.push(c);
                }
            }
            if col_str.is_empty() || row_str.is_empty() {
                return None;
            }
            let col = rust_xlsxwriter::utility::column_name_to_number(&col_str);
            let row = row_str.parse::<u32>().ok()? - 1;
            Some((row, col))
        };

        // Process merged cells first so we don't overwrite data later
        let mut merges = Vec::new();
        for key in &["gabung_sel", "merge_cell", "gabung", "merge"] {
            if let Some(Value::Array(idx)) = sheet_data.get(*key) {
                let arr_cloned = ctx.get_heap_mut().get_array(*idx).clone();
                for item in arr_cloned {
                    if let Value::String(s_idx) = item {
                        merges.push(ctx.get_heap_mut().get_string(s_idx).clone());
                    }
                }
                break;
            }
        }

        if let Some(cells) = cells_map {
            for (cell_ref, cell_val) in cells.iter() {
                let (row, col) = if let Some(parsed) = parse_cell_ref(cell_ref) {
                    parsed
                } else {
                    continue;
                };

                // Check if this cell is the top-left of any merge range
                let mut is_merge_top_left = false;
                let mut merge_r2 = row;
                let mut merge_c2 = col;
                for merge_ref in &merges {
                    let parts: Vec<&str> = merge_ref.split(':').collect();
                    if parts.len() == 2
                        && let (Some((r1, c1)), Some((r2, c2))) =
                            (parse_cell_ref(parts[0]), parse_cell_ref(parts[1]))
                        && row == r1
                        && col == c1
                    {
                        is_merge_top_left = true;
                        merge_r2 = r2;
                        merge_c2 = c2;
                        break;
                    }
                }

                if let Value::Kamus(idx) = cell_val {
                    let props = ctx.get_heap_mut().get_kamus(*idx).clone();
                    let mut format = Format::new();
                    let mut has_format = false;

                    for key in &["tebal", "bold"] {
                        if let Some(Value::Boolean(true)) = props.get(*key) {
                            format = format.set_bold();
                            has_format = true;
                            break;
                        }
                    }

                    for key in &["warna_latar", "bg_color"] {
                        if let Some(Value::String(idx)) = props.get(*key) {
                            let color_str = ctx.get_heap_mut().get_string(*idx).clone();
                            format = format.set_background_color(color_str.as_str());
                            has_format = true;
                            break;
                        }
                    }

                    for key in &["warna_teks", "text_color"] {
                        if let Some(Value::String(idx)) = props.get(*key) {
                            let color_str = ctx.get_heap_mut().get_string(*idx).clone();
                            format = format.set_font_color(color_str.as_str());
                            has_format = true;
                            break;
                        }
                    }

                    let format_opt = if has_format { Some(&format) } else { None };

                    let mut wrote = false;

                    for key in &["rumus", "formula"] {
                        if let Some(Value::String(idx)) = props.get(*key) {
                            let formula = ctx.get_heap_mut().get_string(*idx).clone();
                            if is_merge_top_left {
                                worksheet
                                    .merge_range(
                                        row,
                                        col,
                                        merge_r2,
                                        merge_c2,
                                        formula.as_str(),
                                        format_opt.unwrap_or(&Format::new()),
                                    )
                                    .map_err(|e| e.to_string())?;
                            } else {
                                if let Some(fmt) = format_opt {
                                    worksheet
                                        .write_formula_with_format(row, col, formula.as_str(), fmt)
                                        .map_err(|e| e.to_string())?;
                                } else {
                                    worksheet
                                        .write_formula(row, col, formula.as_str())
                                        .map_err(|e| e.to_string())?;
                                }
                            }
                            wrote = true;
                            break;
                        }
                    }

                    if !wrote {
                        for key in &["angka", "number"] {
                            if let Some(Value::Angka(num)) = props.get(*key) {
                                if is_merge_top_left {
                                    worksheet
                                        .merge_range(
                                            row,
                                            col,
                                            merge_r2,
                                            merge_c2,
                                            &num.to_string(),
                                            format_opt.unwrap_or(&Format::new()),
                                        )
                                        .map_err(|e| e.to_string())?;
                                } else {
                                    if let Some(fmt) = format_opt {
                                        worksheet
                                            .write_number_with_format(row, col, *num, fmt)
                                            .map_err(|e| e.to_string())?;
                                    } else {
                                        worksheet
                                            .write_number(row, col, *num)
                                            .map_err(|e| e.to_string())?;
                                    }
                                }
                                wrote = true;
                                break;
                            }
                        }
                    }

                    if !wrote {
                        for key in &["teks", "text"] {
                            if let Some(Value::String(idx)) = props.get(*key) {
                                let text = ctx.get_heap_mut().get_string(*idx).clone();
                                if is_merge_top_left {
                                    worksheet
                                        .merge_range(
                                            row,
                                            col,
                                            merge_r2,
                                            merge_c2,
                                            text.as_str(),
                                            format_opt.unwrap_or(&Format::new()),
                                        )
                                        .map_err(|e| e.to_string())?;
                                } else {
                                    if let Some(fmt) = format_opt {
                                        worksheet
                                            .write_string_with_format(row, col, text.as_str(), fmt)
                                            .map_err(|e| e.to_string())?;
                                    } else {
                                        worksheet
                                            .write_string(row, col, text.as_str())
                                            .map_err(|e| e.to_string())?;
                                    }
                                }
                                wrote = true;
                                break;
                            }
                        }
                    }

                    if !wrote && is_merge_top_left {
                        // Empty merge range
                        worksheet
                            .merge_range(
                                row,
                                col,
                                merge_r2,
                                merge_c2,
                                "",
                                format_opt.unwrap_or(&Format::new()),
                            )
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        Ok(())
    };

    match sumber {
        Value::Kamus(idx) => {
            let map = ctx.get_heap_mut().get_kamus(*idx).clone();
            process_sheet(&mut workbook, &map, ctx)?;
        }
        Value::Array(idx) => {
            let arr = ctx.get_heap_mut().get_array(*idx).clone();
            for item in arr {
                if let Value::Kamus(d_idx) = item {
                    let map = ctx.get_heap_mut().get_kamus(d_idx).clone();
                    process_sheet(&mut workbook, &map, ctx)?;
                }
            }
        }
        _ => return Err("Sumber untuk tipe excel harus berupa Dictionary atau Array".to_string()),
    }

    workbook.save(final_filename).map_err(|e| e.to_string())?;

    Ok(Value::Kosong)
}

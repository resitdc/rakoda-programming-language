pub fn preprocess_template(input: &str) -> String {
    let mut output = String::new();
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();
    let mut in_ipl = false;

    while i < chars.len() {
        if !in_ipl {
            // Cek apakah masuk blok RPL
            if i + 4 < chars.len() && chars[i..i + 5] == ['<', '?', 'r', 'p', 'l'] {
                in_ipl = true;
                i += 5;
                continue;
            }

            // Cek apakah ada variabel {{ ... }}
            if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                i += 2;
                let mut expr = String::new();
                while i + 1 < chars.len() && !(chars[i] == '}' && chars[i + 1] == '}') {
                    expr.push(chars[i]);
                    i += 1;
                }
                i += 2; // lewati '}}'
                output.push_str(&format!("\ncetak {}\n", expr.trim()));
                continue;
            }

            // Kumpulkan HTML murni
            let mut html_chunk = String::new();
            while i < chars.len() {
                if i + 4 < chars.len() && chars[i..i + 5] == ['<', '?', 'r', 'p', 'l'] {
                    break;
                }
                if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                    break;
                }
                html_chunk.push(chars[i]);
                i += 1;
            }

            if !html_chunk.is_empty() {
                // Escape quotes and backslashes for string literal
                let escaped = html_chunk.replace('\\', "\\\\").replace('"', "\\\"");
                // Also escape newlines so it can be a valid string literal in RPL
                let escaped = escaped.replace('\n', "\\n").replace('\r', "");
                output.push_str(&format!("\ncetak \"{}\"\n", escaped));
            }
        } else {
            // Mode RPL
            let mut ipl_chunk = String::new();
            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == '?' && chars[i + 1] == '>' {
                    break;
                }
                ipl_chunk.push(chars[i]);
                i += 1;
            }
            output.push_str(&ipl_chunk);
            if i + 1 < chars.len() && chars[i] == '?' && chars[i + 1] == '>' {
                i += 2; // lewati '?>'
                in_ipl = false;
            }
        }
    }

    output
}

pub fn preprocess_template_to_function(input: &str) -> String {
    let mut output = String::new();
    output.push_str("kembalikan fungsi(data) {\n");
    output.push_str("  buat _html = \"\"\n");

    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();
    let mut in_ipl = false;

    while i < chars.len() {
        if !in_ipl {
            // Cek apakah masuk blok RPL
            if i + 4 < chars.len() && chars[i..i + 5] == ['<', '?', 'r', 'p', 'l'] {
                in_ipl = true;
                i += 5;
                continue;
            }

            // Cek apakah ada variabel {{ ... }}
            if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                i += 2;
                let mut expr = String::new();
                while i + 1 < chars.len() && !(chars[i] == '}' && chars[i + 1] == '}') {
                    expr.push(chars[i]);
                    i += 1;
                }
                i += 2; // lewati '}}'
                output.push_str(&format!("  _html = _html + ({})\n", expr.trim()));
                continue;
            }
            // Cek tag komponen berdasarkan nama file seperti <header.rpl> atau <header.rpl.html>
            if i < chars.len()
                && chars[i] == '<'
                && i + 1 < chars.len()
                && chars[i + 1] != '/'
                && chars[i + 1] != '?'
            {
                let mut tag_name = String::new();
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '>' && chars[j] != ' ' && chars[j] != '/' {
                    tag_name.push(chars[j]);
                    j += 1;
                }

                // Cari penutup ">"
                let mut k = j;
                while k < chars.len() && chars[k] != '>' {
                    k += 1;
                }

                if k < chars.len()
                    && chars[k] == '>'
                    && (tag_name.ends_with(".rpl") || tag_name.ends_with(".rpl.html"))
                {
                    let mut final_path = tag_name.clone();
                    if final_path.ends_with(".rpl") && !final_path.ends_with(".rpl.html") {
                        final_path.push_str(".html");
                    }

                    let path = if final_path.contains('/') {
                        final_path
                    } else {
                        format!("tampilan/{}", final_path)
                    };

                    output.push_str(&format!(
                        "  _html = _html + web.render(\"{}\", data)\n",
                        path
                    ));
                    i = k + 1;
                    continue;
                }
            }

            // Kumpulkan HTML murni
            let mut html_chunk = String::new();
            while i < chars.len() {
                if i + 4 < chars.len() && chars[i..i + 5] == ['<', '?', 'r', 'p', 'l'] {
                    break;
                }
                // Stop HTML chunk if we see a potential file tag <xxx.rpl>
                if i + 4 < chars.len() && chars[i] == '<' {
                    let _is_tag = false;
                    let mut tag_name = String::new();
                    let mut temp_j = i + 1;
                    while temp_j < chars.len()
                        && chars[temp_j] != '>'
                        && chars[temp_j] != ' '
                        && chars[temp_j] != '/'
                    {
                        tag_name.push(chars[temp_j]);
                        temp_j += 1;
                    }
                    if tag_name.ends_with(".rpl") || tag_name.ends_with(".rpl.html") {
                        break;
                    }
                }
                if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                    break;
                }
                html_chunk.push(chars[i]);
                i += 1;
            }

            if !html_chunk.is_empty() {
                let escaped = html_chunk.replace('\\', "\\\\").replace('"', "\\\"");
                let escaped = escaped.replace('\n', "\\n").replace('\r', "");
                output.push_str(&format!("  _html = _html + \"{}\"\n", escaped));
            }
        } else {
            // Mode RPL
            let mut ipl_chunk = String::new();
            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == '?' && chars[i + 1] == '>' {
                    break;
                }
                ipl_chunk.push(chars[i]);
                i += 1;
            }
            output.push_str(&ipl_chunk);
            if i + 1 < chars.len() && chars[i] == '?' && chars[i + 1] == '>' {
                i += 2; // lewati '?>'
                in_ipl = false;
            }
        }
    }

    output.push_str("\n  kembalikan _html\n");
    output.push_str("}\n");
    output
}

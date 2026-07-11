pub fn preprocess_template(input: &str) -> String {
    let mut output = String::new();
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();
    let mut in_ipl = false;

    while i < chars.len() {
        if !in_ipl {
            // Cek apakah masuk blok RPL
            if i + 4 < chars.len() && chars[i..i+5] == ['<', '?', 'i', 'p', 'l'] {
                in_ipl = true;
                i += 5;
                continue;
            }

            // Cek apakah ada variabel {{ ... }}
            if i + 1 < chars.len() && chars[i] == '{' && chars[i+1] == '{' {
                i += 2;
                let mut expr = String::new();
                while i + 1 < chars.len() && !(chars[i] == '}' && chars[i+1] == '}') {
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
                if i + 4 < chars.len() && chars[i..i+5] == ['<', '?', 'i', 'p', 'l'] {
                    break;
                }
                if i + 1 < chars.len() && chars[i] == '{' && chars[i+1] == '{' {
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
                if i + 1 < chars.len() && chars[i] == '?' && chars[i+1] == '>' {
                    break;
                }
                ipl_chunk.push(chars[i]);
                i += 1;
            }
            output.push_str(&ipl_chunk);
            if i + 1 < chars.len() && chars[i] == '?' && chars[i+1] == '>' {
                i += 2; // lewati '?>'
                in_ipl = false;
            }
        }
    }

    output
}

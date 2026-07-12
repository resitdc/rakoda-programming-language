use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use crate::heap::HeapData;
use std::collections::HashMap;
use std::time::Instant;
use std::io::{Read, Write};
use flate2::write::GzEncoder;
use flate2::Compression;

fn value_to_json(val: &Value, heap: &crate::heap::Heap) -> serde_json::Value {
    match val {
        Value::Kosong => serde_json::Value::Null,
        Value::Angka(n) => serde_json::json!(*n),
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::String(idx) => serde_json::Value::String(heap.get_string(*idx).clone()),
        Value::Array(idx) => {
            let arr = heap.get_array(*idx);
            let json_arr: Vec<serde_json::Value> = arr.iter().map(|v| value_to_json(v, heap)).collect();
            serde_json::Value::Array(json_arr)
        }
        Value::Kamus(idx) => {
            let dict = heap.get_kamus(*idx);
            let mut map = serde_json::Map::new();
            for (k, v) in dict.iter() {
                map.insert(k.clone(), value_to_json(v, heap));
            }
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::String(val.to_string(heap)),
    }
}

pub fn register(vm: &mut VM) {
    let mut web_map = HashMap::new();
    
    let kompresi_func = FungsiBawaanVM {
        nama: "kompresi".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'web.kompresi' membutuhkan 1 argumen (boolean)".to_string());
            }
            let aktif = match args[0] {
                Value::Boolean(b) => b,
                _ => return Err("Argumen kompresi harus berupa boolean".to_string()),
            };
            ctx.get_heap_mut().web_config.kompresi = aktif;
            Ok(Value::Kosong)
        },
    };
    let kompresi_idx = vm.heap.alloc(HeapData::FungsiBawaan(kompresi_func));
    web_map.insert("kompresi".to_string(), Value::FungsiBawaan(kompresi_idx));

    let ratelimit_func = FungsiBawaanVM {
        nama: "ratelimit".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'web.ratelimit' membutuhkan 1 argumen (angka)".to_string());
            }
            let limit = match args[0] {
                Value::Angka(n) => n as u32,
                _ => return Err("Limit harus berupa angka".to_string()),
            };
            ctx.get_heap_mut().web_config.rate_limit = Some(limit);
            Ok(Value::Kosong)
        },
    };
    let ratelimit_idx = vm.heap.alloc(HeapData::FungsiBawaan(ratelimit_func));
    web_map.insert("ratelimit".to_string(), Value::FungsiBawaan(ratelimit_idx));

    let proxy_func = FungsiBawaanVM {
        nama: "proxy".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'web.proxy' membutuhkan 2 argumen (path, target_url)".to_string());
            }
            if let (Value::String(p_idx), Value::String(t_idx)) = (&args[0], &args[1]) {
                let path = ctx.get_heap_mut().get_string(*p_idx).clone();
                let target = ctx.get_heap_mut().get_string(*t_idx).clone();
                ctx.get_heap_mut().web_config.proxies.insert(path, target);
                Ok(Value::Kosong)
            } else {
                Err("Path dan target URL harus berupa string".to_string())
            }
        },
    };
    let proxy_idx = vm.heap.alloc(HeapData::FungsiBawaan(proxy_func));
    web_map.insert("proxy".to_string(), Value::FungsiBawaan(proxy_idx));

    let render_func = FungsiBawaanVM {
        nama: "render".to_string(),
        func: |ctx, args| {
            if args.is_empty() || args.len() > 2 {
                return Err("Fungsi 'web.render' membutuhkan 1 atau 2 argumen (file, data)".to_string());
            }
            
            let file_name = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen pertama harus berupa string (nama file)".to_string()),
            };
            
            let data_arg = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::Kosong
            };
            
            let file_content = match std::fs::read_to_string(&file_name) {
                Ok(content) => content,
                Err(e) => return Err(format!("Gagal membaca file template '{}': {}", file_name, e)),
            };
            
            let template_code = interpreter::template::preprocess_template_to_function(&file_content);
            
            let func_val = ctx.compile_source(&template_code)?;
            
            match ctx.execute_function(func_val, vec![data_arg]) {
                Ok(val) => Ok(val),
                Err(e) => Err(format!("Gagal me-render template: {}", e)),
            }
        },
    };
    let render_idx = vm.heap.alloc(HeapData::FungsiBawaan(render_func));
    web_map.insert("render".to_string(), Value::FungsiBawaan(render_idx));
    web_map.insert("view".to_string(), Value::FungsiBawaan(render_idx));
    web_map.insert("tampilkan_halaman".to_string(), Value::FungsiBawaan(render_idx));

    // HTTP method routes
    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.len() != 2 { return Err("Fungsi 'web.get' membutuhkan 2 argumen (path, handler)".to_string()); }
            let path = args[0].to_string(ctx.get_heap_mut());
            let func_val = match args[1] { Value::Fungsi(idx, env) => Value::Fungsi(idx, env), _ => return Err("Argumen kedua harus berupa fungsi".to_string()), };
            let method_map = ctx.get_heap_mut().web_routes.entry(path).or_insert_with(HashMap::new);
            method_map.insert("GET".to_string(), func_val);
            Ok(Value::Kosong)
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    web_map.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let post_func = FungsiBawaanVM {
        nama: "post".to_string(),
        func: |ctx, args| {
            if args.len() != 2 { return Err("Fungsi 'web.post' membutuhkan 2 argumen (path, handler)".to_string()); }
            let path = args[0].to_string(ctx.get_heap_mut());
            let func_val = match args[1] { Value::Fungsi(idx, env) => Value::Fungsi(idx, env), _ => return Err("Argumen kedua harus berupa fungsi".to_string()), };
            let method_map = ctx.get_heap_mut().web_routes.entry(path).or_insert_with(HashMap::new);
            method_map.insert("POST".to_string(), func_val);
            Ok(Value::Kosong)
        },
    };
    let post_idx = vm.heap.alloc(HeapData::FungsiBawaan(post_func));
    web_map.insert("post".to_string(), Value::FungsiBawaan(post_idx));

    let put_func = FungsiBawaanVM {
        nama: "put".to_string(),
        func: |ctx, args| {
            if args.len() != 2 { return Err("Fungsi 'web.put' membutuhkan 2 argumen (path, handler)".to_string()); }
            let path = args[0].to_string(ctx.get_heap_mut());
            let func_val = match args[1] { Value::Fungsi(idx, env) => Value::Fungsi(idx, env), _ => return Err("Argumen kedua harus berupa fungsi".to_string()), };
            let method_map = ctx.get_heap_mut().web_routes.entry(path).or_insert_with(HashMap::new);
            method_map.insert("PUT".to_string(), func_val);
            Ok(Value::Kosong)
        },
    };
    let put_idx = vm.heap.alloc(HeapData::FungsiBawaan(put_func));
    web_map.insert("put".to_string(), Value::FungsiBawaan(put_idx));

    let delete_func = FungsiBawaanVM {
        nama: "delete".to_string(),
        func: |ctx, args| {
            if args.len() != 2 { return Err("Fungsi 'web.delete' membutuhkan 2 argumen (path, handler)".to_string()); }
            let path = args[0].to_string(ctx.get_heap_mut());
            let func_val = match args[1] { Value::Fungsi(idx, env) => Value::Fungsi(idx, env), _ => return Err("Argumen kedua harus berupa fungsi".to_string()), };
            let method_map = ctx.get_heap_mut().web_routes.entry(path).or_insert_with(HashMap::new);
            method_map.insert("DELETE".to_string(), func_val);
            Ok(Value::Kosong)
        },
    };
    let delete_idx = vm.heap.alloc(HeapData::FungsiBawaan(delete_func));
    web_map.insert("delete".to_string(), Value::FungsiBawaan(delete_idx));

    let patch_func = FungsiBawaanVM {
        nama: "patch".to_string(),
        func: |ctx, args| {
            if args.len() != 2 { return Err("Fungsi 'web.patch' membutuhkan 2 argumen (path, handler)".to_string()); }
            let path = args[0].to_string(ctx.get_heap_mut());
            let func_val = match args[1] { Value::Fungsi(idx, env) => Value::Fungsi(idx, env), _ => return Err("Argumen kedua harus berupa fungsi".to_string()), };
            let method_map = ctx.get_heap_mut().web_routes.entry(path).or_insert_with(HashMap::new);
            method_map.insert("PATCH".to_string(), func_val);
            Ok(Value::Kosong)
        },
    };
    let patch_idx = vm.heap.alloc(HeapData::FungsiBawaan(patch_func));
    web_map.insert("patch".to_string(), Value::FungsiBawaan(patch_idx));

    // web.statis(path, folder)
    let statis_func = FungsiBawaanVM {
        nama: "statis".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'web.statis' membutuhkan 2 argumen (path, folder)".to_string());
            }
            if let (Value::String(p_idx), Value::String(f_idx)) = (&args[0], &args[1]) {
                let path = ctx.get_heap_mut().get_string(*p_idx).clone();
                let folder = ctx.get_heap_mut().get_string(*f_idx).clone();
                ctx.get_heap_mut().web_static_dirs.insert(path, folder);
                Ok(Value::Kosong)
            } else {
                Err("Path dan folder harus berupa string".to_string())
            }
        },
    };
    let statis_idx = vm.heap.alloc(HeapData::FungsiBawaan(statis_func));
    web_map.insert("statis".to_string(), Value::FungsiBawaan(statis_idx));
    
    let jalankan_func = FungsiBawaanVM {
        nama: "jalankan".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'web.jalankan' membutuhkan 1 argumen (port)".to_string());
            }
            let port = match args[0] {
                Value::Angka(n) => n as u16,
                _ => return Err("Port harus berupa angka".to_string()),
            };
            
            let addr = format!("0.0.0.0:{}", port);
            println!("\x1b[32m🚀 Menjalankan Server Web Native RPL di http://{}\x1b[0m", addr);
            
            let kompresi_aktif = ctx.get_heap_mut().web_config.kompresi;
            let rate_limit = ctx.get_heap_mut().web_config.rate_limit;
            let proxies = ctx.get_heap_mut().web_config.proxies.clone();
            let static_dirs = ctx.get_heap_mut().web_static_dirs.clone();
            
            let server = tiny_http::Server::http(&addr)
                .map_err(|e| format!("Gagal menjalankan server: {}", e))?;
                
            let mut rate_limits: HashMap<String, (Instant, u32)> = HashMap::new();
                
            'req_loop: for mut request in server.incoming_requests() {
                let full_url = request.url().to_string();
                let method = request.method().as_str().to_string();
                
                let (url, query_string) = match full_url.split_once('?') {
                    Some((u, q)) => (u.to_string(), q.to_string()),
                    None => (full_url.clone(), String::new()),
                };
                
                let mut query_params = HashMap::new();
                for pair in query_string.split('&') {
                    if pair.is_empty() { continue; }
                    if let Some((k, v)) = pair.split_once('=') {
                        query_params.insert(k.to_string(), v.to_string());
                    } else {
                        query_params.insert(pair.to_string(), "".to_string());
                    }
                }
                
                // --- AWAL REQUEST: Bersihkan state & parse cookies ---
                ctx.get_heap_mut().web_state.active_cookies.clear();
                ctx.get_heap_mut().web_state.cookies_to_set.clear();
                ctx.get_heap_mut().web_state.active_session_id = None;
                
                for header in request.headers() {
                    if header.field.equiv("Cookie") {
                        let cookie_str = header.value.as_str();
                        for part in cookie_str.split(';') {
                            let part = part.trim();
                            if let Some((k, v)) = part.split_once('=') {
                                ctx.get_heap_mut().web_state.active_cookies.insert(k.to_string(), v.to_string());
                            }
                        }
                    }
                }
                
                if let Some(sid) = ctx.get_heap_mut().web_state.active_cookies.get("RPL_SESSIONID") {
                    ctx.get_heap_mut().web_state.active_session_id = Some(sid.clone());
                }
                // --- AKHIR PARSING COOKIE ---
                
                // 1. Rate Limiting
                if let Some(limit) = rate_limit {
                    let ip = request.remote_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
                    let now = Instant::now();
                    let entry = rate_limits.entry(ip).or_insert((now, 0));
                    if now.duration_since(entry.0).as_secs() < 1 {
                        entry.1 += 1;
                        if entry.1 > limit {
                            let resp = tiny_http::Response::from_string("Too Many Requests").with_status_code(429);
                            let _ = request.respond(resp);
                            continue;
                        }
                    } else {
                        entry.0 = now;
                        entry.1 = 1;
                    }
                }
                
                // 2. Reverse Proxy
                for (prefix, target) in &proxies {
                    if url.starts_with(prefix) {
                        let mut target_url = target.clone();
                        if !target_url.ends_with('/') && !url[prefix.len()..].starts_with('/') && !url[prefix.len()..].is_empty() {
                            target_url.push('/');
                        }
                        target_url.push_str(&url[prefix.len()..]);
                        
                        let mut body = String::new();
                        let _ = request.as_reader().read_to_string(&mut body);

                        let res = match method.as_str() {
                            "POST" => if body.is_empty() { ureq::post(&target_url).send_empty() } else { ureq::post(&target_url).send(body) },
                            "PUT" => if body.is_empty() { ureq::put(&target_url).send_empty() } else { ureq::put(&target_url).send(body) },
                            "PATCH" => if body.is_empty() { ureq::patch(&target_url).send_empty() } else { ureq::patch(&target_url).send(body) },
                            "DELETE" => ureq::delete(&target_url).call(),
                            _ => ureq::get(&target_url).call(),
                        };
                        match res {
                            Ok(mut resp) => {
                                let status = resp.status().as_u16();
                                let mut resp_body = String::new();
                                let _ = resp.body_mut().as_reader().read_to_string(&mut resp_body);
                                let tiny_resp = tiny_http::Response::from_string(resp_body).with_status_code(status);
                                let _ = request.respond(tiny_resp);
                            }
                            Err(e) => {
                                let status = match e {
                                    ureq::Error::StatusCode(code) => code,
                                    _ => 502,
                                };
                                let tiny_resp = tiny_http::Response::from_string("Bad Gateway").with_status_code(status);
                                let _ = request.respond(tiny_resp);
                            }
                        }
                        continue 'req_loop;
                    }
                }
                
                // 3. Static File Serving
                let mut static_response = None;
                for (prefix, folder) in &static_dirs {
                    if url.starts_with(prefix) {
                        let file_path = url[prefix.len()..].trim_start_matches('/');
                        let full_path = std::path::Path::new(folder).join(file_path);
                        
                        if full_path.exists() && full_path.is_file() {
                            if let Ok(content) = std::fs::read(&full_path) {
                                let content_type = match full_path.extension().and_then(|e| e.to_str()) {
                                    Some("html") => "text/html; charset=utf-8",
                                    Some("css") => "text/css; charset=utf-8",
                                    Some("js") => "application/javascript; charset=utf-8",
                                    Some("json") => "application/json; charset=utf-8",
                                    Some("png") => "image/png",
                                    Some("jpg") | Some("jpeg") => "image/jpeg",
                                    Some("gif") => "image/gif",
                                    Some("svg") => "image/svg+xml",
                                    Some("ico") => "image/x-icon",
                                    Some("woff") => "font/woff",
                                    Some("woff2") => "font/woff2",
                                    Some("ttf") => "font/ttf",
                                    _ => "application/octet-stream",
                                };
                                static_response = Some((content, content_type));
                                break;
                            }
                        }
                    }
                }
                
                if let Some((content, content_type)) = static_response {
                    let mut resp = tiny_http::Response::from_data(content);
                    resp.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
                    let _ = request.respond(resp);
                    continue 'req_loop;
                }
                
                // 4. Dynamic Routing (with parameter support)
                let (route_opt, params) = find_route(ctx.get_heap_mut(), &url, &method);
                
                match route_opt {
                    Some(func_val) => {
                        let req_kamus_idx = {
                            let mut req_map = HashMap::new();
                            let url_str = ctx.get_heap_mut().alloc(HeapData::String(url.clone()));
                            req_map.insert("url".to_string(), Value::String(url_str));
                            
                            let method_str = ctx.get_heap_mut().alloc(HeapData::String(method.clone()));
                            req_map.insert("metode".to_string(), Value::String(method_str));
                            
                            let mut body = String::new();
                            let _ = request.as_reader().read_to_string(&mut body);
                            let body_str = ctx.get_heap_mut().alloc(HeapData::String(body.clone()));
                            req_map.insert("tubuh".to_string(), Value::String(body_str));
                            
                            // Cek header Content-Type
                            let mut is_json = false;
                            for header in request.headers() {
                                if header.field.equiv("Content-Type") {
                                    if header.value.as_str().contains("application/json") {
                                        is_json = true;
                                        break;
                                    }
                                }
                            }
                            
                            // Auto parsing JSON
                            if is_json && !body.is_empty() {
                                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body) {
                                    let rpl_val = crate::stdlib::json::convert_to_value(ctx, &json_val);
                                    req_map.insert("json".to_string(), rpl_val);
                                }
                            }
                            
                            // Add kueri
                            let mut kueri_map = HashMap::new();
                            for (k, v) in &query_params {
                                let v_idx = ctx.get_heap_mut().alloc(HeapData::String(v.clone()));
                                kueri_map.insert(k.clone(), Value::String(v_idx));
                            }
                            let kueri_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(kueri_map));
                            req_map.insert("kueri".to_string(), Value::Kamus(kueri_idx));
                            
                            // Add params
                            if !params.is_empty() {
                                let mut params_map = HashMap::new();
                                for (k, v) in params {
                                    let v_idx = ctx.get_heap_mut().alloc(HeapData::String(v));
                                    params_map.insert(k, Value::String(v_idx));
                                }
                                let params_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(params_map));
                                req_map.insert("params".to_string(), Value::Kamus(params_idx));
                            }
                            
                            ctx.get_heap_mut().alloc(HeapData::Kamus(req_map))
                        };
                        
                        let req_val = Value::Kamus(req_kamus_idx);
                        let hasil = ctx.execute_function(func_val, vec![req_val]);
                        
                        match hasil {
                            Ok(val) => {
                                let mut response_status = 200;
                                let mut val_string = String::new();
                                let mut content_type = "text/html";

                                if let Value::Kamus(idx) = val {
                                    let dict = ctx.get_heap_mut().get_kamus(idx).clone();
                                    if dict.contains_key("status") && (dict.contains_key("json") || dict.contains_key("tubuh")) {
                                        if let Some(Value::Angka(s)) = dict.get("status") {
                                            response_status = *s as u16;
                                        }
                                        if let Some(json_val) = dict.get("json") {
                                            val_string = value_to_json(json_val, ctx.get_heap_mut()).to_string();
                                            content_type = "application/json";
                                        } else if let Some(Value::String(s_idx)) = dict.get("tubuh") {
                                            val_string = ctx.get_heap_mut().get_string(*s_idx).clone();
                                        } else if let Some(v) = dict.get("tubuh") {
                                            val_string = v.to_string(ctx.get_heap_mut());
                                        }
                                    } else {
                                        val_string = value_to_json(&val, ctx.get_heap_mut()).to_string();
                                        content_type = "application/json";
                                    }
                                } else if let Value::Array(_) = val {
                                    val_string = value_to_json(&val, ctx.get_heap_mut()).to_string();
                                    content_type = "application/json";
                                } else {
                                    val_string = val.to_string(ctx.get_heap_mut());
                                }
                                
                                let mut accept_encoding = String::new();
                                for h in request.headers() {
                                    if h.field.equiv("Accept-Encoding") {
                                        accept_encoding = h.value.as_str().to_lowercase();
                                        break;
                                    }
                                }
                                
                                let mut response = if kompresi_aktif && accept_encoding.contains("br") {
                                    let mut compressed = Vec::new();
                                    let mut cursor = std::io::Cursor::new(val_string.as_bytes());
                                    let _ = brotli::CompressorReader::new(&mut cursor, 4096, 11, 22).read_to_end(&mut compressed);
                                    let mut r = tiny_http::Response::from_data(compressed);
                                    r.add_header(tiny_http::Header::from_bytes(&b"Content-Encoding"[..], &b"br"[..]).unwrap());
                                    r
                                } else if kompresi_aktif && accept_encoding.contains("gzip") {
                                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                                    let _ = encoder.write_all(val_string.as_bytes());
                                    let compressed = encoder.finish().unwrap_or_default();
                                    let mut r = tiny_http::Response::from_data(compressed);
                                    r.add_header(tiny_http::Header::from_bytes(&b"Content-Encoding"[..], &b"gzip"[..]).unwrap());
                                    r
                                } else {
                                    tiny_http::Response::from_string(val_string)
                                }.with_status_code(response_status);
                                
                                response.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
                                
                                let cookies = ctx.get_heap_mut().web_state.cookies_to_set.clone();
                                for cookie in cookies {
                                    response.add_header(tiny_http::Header::from_bytes(&b"Set-Cookie"[..], cookie.as_bytes()).unwrap());
                                }
                                
                                let _ = request.respond(response);
                            }
                            Err(e) => {
                                let err_resp = tiny_http::Response::from_string(format!("Internal Server Error: {}", e))
                                    .with_status_code(500);
                                let _ = request.respond(err_resp);
                            }
                        }
                    }
                    None => {
                        let resp = tiny_http::Response::from_string("Not Found").with_status_code(404);
                        let _ = request.respond(resp);
                    }
                }
            }
            Ok(Value::Kosong)
        },
    };
    let jalankan_idx = vm.heap.alloc(HeapData::FungsiBawaan(jalankan_func));
    web_map.insert("jalankan".to_string(), Value::FungsiBawaan(jalankan_idx));
    
    let web_idx = vm.heap.alloc(HeapData::Kamus(web_map));
    vm.set_global("web".to_string(), Value::Kamus(web_idx));
}

// Find a matching route, supporting :param dynamic segments
fn find_route(heap: &crate::heap::Heap, url: &str, method: &str) -> (Option<Value>, HashMap<String, String>) {
    // Try exact match first
    if let Some(method_map) = heap.web_routes.get(url) {
        if let Some(func) = method_map.get(method) {
            return (Some(*func), HashMap::new());
        }
    }
    
    // Try pattern match with :param
    let url_parts: Vec<&str> = url.trim_matches('/').split('/').collect();
    
    for (pattern, method_map) in &heap.web_routes {
        if !pattern.contains(':') { continue; }
        
        let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
        if pattern_parts.len() != url_parts.len() { continue; }
        
        let mut params = HashMap::new();
        let mut matched = true;
        
        for (pp, up) in pattern_parts.iter().zip(url_parts.iter()) {
            if pp.starts_with(':') {
                params.insert(pp[1..].to_string(), up.to_string());
            } else if pp != up {
                matched = false;
                break;
            }
        }
        
        if matched {
            if let Some(func) = method_map.get(method) {
                return (Some(*func), params);
            }
        }
    }
    
    (None, HashMap::new())
}

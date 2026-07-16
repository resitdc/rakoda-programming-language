use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;
use std::io::Read;

fn value_to_json(val: &Value, heap: &crate::heap::Heap) -> serde_json::Value {
    match val {
        Value::Kosong => serde_json::Value::Null,
        Value::Angka(n) => serde_json::json!(*n),
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::String(idx) => serde_json::Value::String(heap.get_string(*idx).clone()),
        Value::Array(idx) => {
            let arr = heap.get_array(*idx);
            let json_arr: Vec<serde_json::Value> =
                arr.iter().map(|v| value_to_json(v, heap)).collect();
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
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
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
        ),
    };
    let kompresi_idx = vm.heap.alloc(HeapData::FungsiBawaan(kompresi_func));
    web_map.insert("kompresi".to_string(), Value::FungsiBawaan(kompresi_idx));

    let ratelimit_func = FungsiBawaanVM {
        nama: "ratelimit".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
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
        ),
    };
    let ratelimit_idx = vm.heap.alloc(HeapData::FungsiBawaan(ratelimit_func));
    web_map.insert("ratelimit".to_string(), Value::FungsiBawaan(ratelimit_idx));

    let proxy_func = FungsiBawaanVM {
        nama: "proxy".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.proxy' membutuhkan 2 argumen (path, target_url)".to_string(),
                    );
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
        ),
    };
    let proxy_idx = vm.heap.alloc(HeapData::FungsiBawaan(proxy_func));
    web_map.insert("proxy".to_string(), Value::FungsiBawaan(proxy_idx));

    let render_func = FungsiBawaanVM {
        nama: "render".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.is_empty() || args.len() > 2 {
                    return Err(
                        "Fungsi 'web.render' membutuhkan 1 atau 2 argumen (file, data)".to_string(),
                    );
                }

                let raw_file_name = match &args[0] {
                    Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                    _ => return Err("Argumen pertama harus berupa string (nama file)".to_string()),
                };

                // Resolve path relatif terhadap project_root (direktori file sumber),
                // bukan CWD (current working directory).
                let file_name = if std::path::Path::new(&raw_file_name).is_relative() {
                    if let Some(root) = &ctx.get_heap_mut().project_root {
                        root.join(&raw_file_name).to_string_lossy().to_string()
                    } else {
                        raw_file_name
                    }
                } else {
                    raw_file_name
                };

                let data_arg = if args.len() == 2 {
                    args[1]
                } else {
                    Value::Kosong
                };
                let template_code = if let Some(cached_code) = ctx
                    .get_heap_mut()
                    .web_cache
                    .lock()
                    .unwrap()
                    .templates_code
                    .get(&file_name)
                    .cloned()
                {
                    cached_code
                } else {
                    let file_content = match std::fs::read_to_string(&file_name) {
                        Ok(content) => content,
                        Err(e) => {
                            return Err(format!(
                                "Gagal membaca file template '{}': {}",
                                file_name, e
                            ));
                        }
                    };

                    let code = stdlib::template::preprocess_template_to_function(&file_content);
                    ctx.get_heap_mut()
                        .web_cache
                        .lock()
                        .unwrap()
                        .templates_code
                        .insert(file_name.clone(), code.clone());
                    code
                };

                let func_val = ctx.compile_source(&template_code)?;

                match ctx.execute_function(func_val, vec![data_arg]) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(format!("Gagal me-render template: {}", e)),
                }
            },
        ),
    };
    let render_idx = vm.heap.alloc(HeapData::FungsiBawaan(render_func));
    web_map.insert("render".to_string(), Value::FungsiBawaan(render_idx));
    web_map.insert("view".to_string(), Value::FungsiBawaan(render_idx));
    web_map.insert(
        "tampilkan_halaman".to_string(),
        Value::FungsiBawaan(render_idx),
    );

    // HTTP method routes
    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.get' membutuhkan 2 argumen (path, handler)".to_string()
                    );
                }
                let path = args[0].to_string(ctx.get_heap_mut());
                let func_val = match args[1] {
                    Value::Fungsi(idx, env) => Value::Fungsi(idx, env),
                    _ => return Err("Argumen kedua harus berupa fungsi".to_string()),
                };
                let method_map = ctx.get_heap_mut().web_routes.entry(path).or_default();
                method_map.insert("GET".to_string(), func_val);
                Ok(Value::Kosong)
            },
        ),
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    web_map.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let post_func = FungsiBawaanVM {
        nama: "post".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.post' membutuhkan 2 argumen (path, handler)".to_string()
                    );
                }
                let path = args[0].to_string(ctx.get_heap_mut());
                let func_val = match args[1] {
                    Value::Fungsi(idx, env) => Value::Fungsi(idx, env),
                    _ => return Err("Argumen kedua harus berupa fungsi".to_string()),
                };
                let method_map = ctx.get_heap_mut().web_routes.entry(path).or_default();
                method_map.insert("POST".to_string(), func_val);
                Ok(Value::Kosong)
            },
        ),
    };
    let post_idx = vm.heap.alloc(HeapData::FungsiBawaan(post_func));
    web_map.insert("post".to_string(), Value::FungsiBawaan(post_idx));

    let put_func = FungsiBawaanVM {
        nama: "put".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.put' membutuhkan 2 argumen (path, handler)".to_string()
                    );
                }
                let path = args[0].to_string(ctx.get_heap_mut());
                let func_val = match args[1] {
                    Value::Fungsi(idx, env) => Value::Fungsi(idx, env),
                    _ => return Err("Argumen kedua harus berupa fungsi".to_string()),
                };
                let method_map = ctx.get_heap_mut().web_routes.entry(path).or_default();
                method_map.insert("PUT".to_string(), func_val);
                Ok(Value::Kosong)
            },
        ),
    };
    let put_idx = vm.heap.alloc(HeapData::FungsiBawaan(put_func));
    web_map.insert("put".to_string(), Value::FungsiBawaan(put_idx));

    let delete_func = FungsiBawaanVM {
        nama: "delete".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.delete' membutuhkan 2 argumen (path, handler)".to_string()
                    );
                }
                let path = args[0].to_string(ctx.get_heap_mut());
                let func_val = match args[1] {
                    Value::Fungsi(idx, env) => Value::Fungsi(idx, env),
                    _ => return Err("Argumen kedua harus berupa fungsi".to_string()),
                };
                let method_map = ctx.get_heap_mut().web_routes.entry(path).or_default();
                method_map.insert("DELETE".to_string(), func_val);
                Ok(Value::Kosong)
            },
        ),
    };
    let delete_idx = vm.heap.alloc(HeapData::FungsiBawaan(delete_func));
    web_map.insert("delete".to_string(), Value::FungsiBawaan(delete_idx));

    let patch_func = FungsiBawaanVM {
        nama: "patch".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.patch' membutuhkan 2 argumen (path, handler)".to_string()
                    );
                }
                let path = args[0].to_string(ctx.get_heap_mut());
                let func_val = match args[1] {
                    Value::Fungsi(idx, env) => Value::Fungsi(idx, env),
                    _ => return Err("Argumen kedua harus berupa fungsi".to_string()),
                };
                let method_map = ctx.get_heap_mut().web_routes.entry(path).or_default();
                method_map.insert("PATCH".to_string(), func_val);
                Ok(Value::Kosong)
            },
        ),
    };
    let patch_idx = vm.heap.alloc(HeapData::FungsiBawaan(patch_func));
    web_map.insert("patch".to_string(), Value::FungsiBawaan(patch_idx));

    // web.statis(path, folder)
    let statis_func = FungsiBawaanVM {
        nama: "statis".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 2 {
                    return Err(
                        "Fungsi 'web.statis' membutuhkan 2 argumen (path, folder)".to_string()
                    );
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
        ),
    };
    let statis_idx = vm.heap.alloc(HeapData::FungsiBawaan(statis_func));
    web_map.insert("statis".to_string(), Value::FungsiBawaan(statis_idx));

    let jalankan_func = FungsiBawaanVM {
        nama: "jalankan".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() != 1 {
                    return Err("Fungsi 'web.jalankan' membutuhkan 1 argumen (port)".to_string());
                }
                let port = match args[0] {
                    Value::Angka(n) => n as u16,
                    _ => return Err("Port harus berupa angka".to_string()),
                };

                let addr = format!("0.0.0.0:{}", port);

                let _kompresi_aktif = ctx.get_heap_mut().web_config.kompresi;

                let app = axum::Router::new()
                .route("/__dev", axum::routing::get(|| async {
                    if crate::stdlib::dev_dashboard::is_dev_mode() {
                        axum::response::Response::builder()
                            .header("Content-Type", "text/html")
                            .body(axum::body::Body::from(crate::stdlib::dev_dashboard::DASHBOARD_HTML))
                            .unwrap()
                    } else {
                        axum::response::Response::builder().status(404).body(axum::body::Body::from("Not Found")).unwrap()
                    }
                }))
                .route("/___dev_ws", axum::routing::get(dev_ws_handler))
                .fallback(axum::routing::any(
                |axum::extract::State(vm_state): axum::extract::State<std::sync::Arc<std::sync::Mutex<crate::machine::VM>>>,
                 req: axum::extract::Request| async move {

                    // VM akan di-lock di dalam spawn_blocking untuk thread safety

                    let method = req.method().as_str().to_string();
                    let uri = req.uri().clone();
                    let url = uri.path().to_string();
                    let query_string = uri.query().unwrap_or("").to_string();

                    let mut query_params = HashMap::new();
                    for pair in query_string.split('&') {
                        if pair.is_empty() { continue; }
                        if let Some((k, v)) = pair.split_once('=') {
                            query_params.insert(k.to_string(), v.to_string());
                        } else {
                            query_params.insert(pair.to_string(), "".to_string());
                        }
                    }

                    let mut headers_map = HashMap::new();
                    let mut is_json = false;
                    let mut multipart_boundary = None;

                    for (k, v) in req.headers() {
                        let header_val = v.to_str().unwrap_or("").to_string();
                        headers_map.insert(k.as_str().to_string(), header_val.clone());

                        if k.as_str().eq_ignore_ascii_case("content-type") {
                            if header_val.contains("application/json") {
                                is_json = true;
                            } else if header_val.contains("multipart/form-data")
                                && let Some(idx) = header_val.find("boundary=") {
                                    multipart_boundary = Some(header_val[idx + 9..].to_string());
                                }
                        }
                    }

                    let mut active_cookies = HashMap::new();
                    let mut active_session_id = None;
                    if let Some(cookie_str) = headers_map.get("cookie") {
                        for part in cookie_str.split(';') {
                            let part = part.trim();
                            if let Some((k, v)) = part.split_once('=') {
                                active_cookies.insert(k.to_string(), v.to_string());
                                if k == "RPL_SESSIONID" {
                                    active_session_id = Some(v.to_string());
                                }
                            }
                        }
                    }

                    // Convert body stream to bytes
                    let body_bytes = axum::body::to_bytes(req.into_body(), 1024 * 1024 * 100).await.unwrap_or_default();
                    let raw_body = body_bytes.to_vec();

                    // --- Eksekusi Synchronous RPL di Thread Pool (Spawn Blocking) ---
                    let result = tokio::task::spawn_blocking(move || -> Result<axum::response::Response, String> {
                        let start = std::time::Instant::now();
                        let mut local_vm = vm_state.lock().unwrap();
                        local_vm.heap.web_state.active_cookies = active_cookies;
                        local_vm.heap.web_state.cookies_to_set.clear();
                        local_vm.heap.web_state.active_session_id = active_session_id;

                        let static_dirs = local_vm.heap.web_static_dirs.clone();
                        let _proxies = local_vm.heap.web_config.proxies.clone();

                        // 1. Static File Serving
                        for (prefix, folder) in &static_dirs {
                            if url.starts_with(prefix) {
                                let file_path = url[prefix.len()..].trim_start_matches('/');
                                let full_path = std::path::Path::new(folder).join(file_path);

                                if full_path.exists() && full_path.is_file() {
                                    let path_str = full_path.to_string_lossy().to_string();

                                    let cache_hit = {
                                        let cache = local_vm.heap.web_cache.lock().unwrap();
                                        cache.static_files.get(&path_str).cloned()
                                    };

                                    if let Some((content_type, content)) = cache_hit {
                                        return Ok(axum::response::Response::builder()
                                            .status(200)
                                            .header("Content-Type", content_type)
                                            .body(axum::body::Body::from(content))
                                            .unwrap());
                                    }

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

                                        let mut cache = local_vm.heap.web_cache.lock().unwrap();
                                        cache.static_files.insert(path_str, (content_type.to_string(), content.clone()));

                                        return Ok(axum::response::Response::builder()
                                            .status(200)
                                            .header("Content-Type", content_type)
                                            .body(axum::body::Body::from(content))
                                            .unwrap());
                                    }
                                }
                            }
                        }

                        // 2. Dynamic Routing
                        let (route_opt, params) = find_route(&local_vm.heap, &url, &method);

                        match route_opt {
                            Some(func_val) => {
                                let req_kamus_idx = {
                                    let mut req_map = HashMap::new();
                                    let url_str = local_vm.heap.alloc(HeapData::String(url.clone()));
                                    req_map.insert("url".to_string(), Value::String(url_str));

                                    let method_str = local_vm.heap.alloc(HeapData::String(method.clone()));
                                    req_map.insert("metode".to_string(), Value::String(method_str));

                                    let mut form_map = HashMap::new();
                                    let mut file_map = HashMap::new();

                                    if let Some(boundary) = multipart_boundary {
                                        let mut cursor = std::io::Cursor::new(raw_body.clone());
                                        let mut multipart = multipart::server::Multipart::with_body(&mut cursor, &boundary);
                                        let tmp_dir = ".rpl_tmp";
                                        let _ = std::fs::create_dir_all(tmp_dir);

                                        while let Ok(Some(mut field)) = multipart.read_entry() {
                                            let name = field.headers.name.to_string();
                                            if let Some(fname) = field.headers.filename {
                                                let unique_name = format!("{}_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(), fname);
                                                let path = format!("{}/{}", tmp_dir, unique_name);

                                                if let Ok(mut file) = std::fs::File::create(&path)
                                                    && std::io::copy(&mut field.data, &mut file).is_ok() {
                                                        let mut file_info = HashMap::new();
                                                        let ukuran = std::fs::metadata(&path).map(|m| m.len() as f64).unwrap_or(0.0);

                                                        let nama_idx = local_vm.heap.alloc(HeapData::String(fname));
                                                        file_info.insert("nama".to_string(), Value::String(nama_idx));

                                                        let path_idx = local_vm.heap.alloc(HeapData::String(path));
                                                        file_info.insert("path".to_string(), Value::String(path_idx));

                                                        file_info.insert("ukuran".to_string(), Value::Angka(ukuran));

                                                        if let Some(ct) = field.headers.content_type {
                                                            let tipe_idx = local_vm.heap.alloc(HeapData::String(ct.to_string()));
                                                            file_info.insert("tipe".to_string(), Value::String(tipe_idx));
                                                        }

                                                        let info_idx = local_vm.heap.alloc(HeapData::Kamus(file_info));
                                                        file_map.insert(name, Value::Kamus(info_idx));
                                                    }
                                            } else {
                                                let mut text_val = String::new();
                                                if field.data.read_to_string(&mut text_val).is_ok() {
                                                    let str_idx = local_vm.heap.alloc(HeapData::String(text_val));
                                                    form_map.insert(name, Value::String(str_idx));
                                                }
                                            }
                                        }
                                        let body_str = local_vm.heap.alloc(HeapData::String(String::new()));
                                        req_map.insert("tubuh".to_string(), Value::String(body_str));
                                        req_map.insert("tubuh_mentah".to_string(), Value::String(body_str));
                                    } else {
                                        let body_string = String::from_utf8_lossy(&raw_body).to_string();
                                        let body_str = local_vm.heap.alloc(HeapData::String(body_string.clone()));
                                        req_map.insert("tubuh_mentah".to_string(), Value::String(body_str));

                                        if is_json && !body_string.is_empty()
                                            && let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body_string) {
                                                let rpl_val = crate::stdlib::json::convert_to_value(&mut local_vm, &json_val);
                                                req_map.insert("tubuh".to_string(), rpl_val);
                                                req_map.insert("json".to_string(), rpl_val);
                                            } else {
                                                req_map.insert("tubuh".to_string(), Value::String(body_str));
                                            }
                                    }

                                    let form_idx = local_vm.heap.alloc(HeapData::Kamus(form_map));
                                    req_map.insert("form".to_string(), Value::Kamus(form_idx));

                                    let file_idx = local_vm.heap.alloc(HeapData::Kamus(file_map));
                                    req_map.insert("file".to_string(), Value::Kamus(file_idx));

                                    let mut kueri_map = HashMap::new();
                                    for (k, v) in &query_params {
                                        let v_idx = local_vm.heap.alloc(HeapData::String(v.clone()));
                                        kueri_map.insert(k.clone(), Value::String(v_idx));
                                    }
                                    let kueri_idx = local_vm.heap.alloc(HeapData::Kamus(kueri_map));
                                    req_map.insert("kueri".to_string(), Value::Kamus(kueri_idx));

                                    if !params.is_empty() {
                                        let mut params_map = HashMap::new();
                                        for (k, v) in &params {
                                            let v_idx = local_vm.heap.alloc(HeapData::String(v.clone()));
                                            params_map.insert(k.clone(), Value::String(v_idx));
                                        }
                                        let params_idx = local_vm.heap.alloc(HeapData::Kamus(params_map));
                                        req_map.insert("params".to_string(), Value::Kamus(params_idx));
                                    }

                                    local_vm.heap.alloc(HeapData::Kamus(req_map))
                                };

                                let req_val = Value::Kamus(req_kamus_idx);
                                let hasil = local_vm.execute_function(func_val, vec![req_val]);

                                match hasil {
                                    Ok(val) => {
                                        let mut response_status = 200;
                                        let mut val_string = String::new();
                                        let mut content_type = "text/html";

                                        if let Value::Kamus(idx) = val {
                                            let dict = local_vm.heap.get_kamus(idx).clone();
                                            if dict.contains_key("status") && (dict.contains_key("json") || dict.contains_key("tubuh")) {
                                                if let Some(Value::Angka(s)) = dict.get("status") {
                                                    response_status = *s as u16;
                                                }
                                                if let Some(json_val) = dict.get("json") {
                                                    val_string = value_to_json(json_val, &local_vm.heap).to_string();
                                                    content_type = "application/json";
                                                } else if let Some(Value::String(s_idx)) = dict.get("tubuh") {
                                                    val_string = local_vm.heap.get_string(*s_idx).clone();
                                                } else if let Some(v) = dict.get("tubuh") {
                                                    val_string = v.to_string(&local_vm.heap);
                                                }
                                            } else {
                                                val_string = value_to_json(&val, &local_vm.heap).to_string();
                                                content_type = "application/json";
                                            }
                                        } else if let Value::Array(_) = val {
                                            val_string = value_to_json(&val, &local_vm.heap).to_string();
                                            content_type = "application/json";
                                        } else {
                                            val_string = val.to_string(&local_vm.heap);
                                        }

                                        let mut builder = axum::response::Response::builder()
                                            .status(response_status)
                                            .header("Content-Type", content_type);

                                        let cookies_to_set = local_vm.heap.web_state.cookies_to_set.clone();
                                        for cookie in cookies_to_set {
                                            builder = builder.header("Set-Cookie", cookie);
                                        }

                                        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                                        let memory_used_kb = (local_vm.heap.allocated_count * 1024) / 1000;
                                        let response_size = val_string.len();

                                        let telemetry = crate::stdlib::dev_dashboard::RequestTelemetry {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f").to_string(),
                                            method: method.clone(),
                                            url: url.clone(),
                                            status: response_status,
                                            duration_ms,
                                            memory_used_kb,
                                            response_size,
                                            ip: "127.0.0.1".to_string(),
                                            user_agent: headers_map.get("user-agent").cloned().unwrap_or_default(),
                                            request_headers: headers_map.clone(),
                                            response_headers: {
                                                let mut m = HashMap::new();
                                                m.insert("Content-Type".to_string(), content_type.to_string());
                                                m
                                            },
                                            raw_body: String::from_utf8_lossy(&raw_body).to_string(),
                                            query_params: query_params.clone(),
                                            route_params: params.clone(),
                                            lifecycle_events: vec![
                                                crate::stdlib::dev_dashboard::LifecycleEvent {
                                                    name: "routing".to_string(),
                                                    duration_ms: duration_ms * 0.1,
                                                    memory_used_kb: 0,
                                                },
                                                crate::stdlib::dev_dashboard::LifecycleEvent {
                                                    name: "controller".to_string(),
                                                    duration_ms: duration_ms * 0.6,
                                                    memory_used_kb,
                                                },
                                                crate::stdlib::dev_dashboard::LifecycleEvent {
                                                    name: "render".to_string(),
                                                    duration_ms: duration_ms * 0.3,
                                                    memory_used_kb: 0,
                                                },
                                            ],
                                        };
                                        crate::stdlib::dev_dashboard::record_request(telemetry);

                                        Ok(builder.body(axum::body::Body::from(val_string)).unwrap())
                                    }
                                    Err(e) => {
                                        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>RPL Error: 500</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background-color: #222; margin: 0; padding: 2rem; color: #fff; }}
        .container {{ background-color: #fff; color: #1f2937; padding: 2rem; border-radius: 8px; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); max-width: 800px; margin: 0 auto; }}
        h1 {{ color: #dc2626; margin-top: 0; display: flex; align-items: center; gap: 10px; font-size: 1.8rem; border-bottom: 2px solid #fee2e2; padding-bottom: 1rem; }}
        .req-info {{ background-color: #f3f4f6; padding: 0.5rem 1rem; border-radius: 6px; font-family: monospace; font-size: 1rem; color: #374151; display: inline-block; margin-bottom: 1.5rem; border: 1px solid #d1d5db; }}
        .error-message {{ background-color: #fee2e2; border-left: 4px solid #dc2626; padding: 1rem; font-family: monospace; font-size: 1rem; overflow-x: auto; white-space: pre-wrap; line-height: 1.5; border-radius: 0 4px 4px 0; color: #991b1b;}}
        .footer {{ margin-top: 2rem; font-size: 0.875rem; color: #6b7280; text-align: center; border-top: 1px solid #e5e7eb; padding-top: 1rem; }}
        .brand {{ font-weight: bold; color: #ef4444; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Terjadi Kesalahan Internal Server (500)</h1>
        <p>Aplikasi Anda mengalami masalah saat memproses request berikut:</p>
        <div class="req-info">{} {}</div>

        <h3 style="margin-bottom: 0.5rem; color: #1f2937;">Pesan Error:</h3>
        <div class="error-message">{}</div>

        <div class="footer">
            <span class="brand">Rakoda Programming Language (RPL)</span> Web Framework
        </div>
    </div>
</body>
</html>"#, method, url, e);
                                        Ok(axum::response::Response::builder().status(500).header("Content-Type", "text/html").body(axum::body::Body::from(html)).unwrap())
                                    }
                                }
                            }
                            None => {
                                Ok(axum::response::Response::builder().status(404).body(axum::body::Body::from("Not Found")).unwrap())
                            }
                        }
                    }).await.unwrap();

                    match result {
                        Ok(resp) => resp,
                        Err(_) => axum::response::Response::builder().status(500).body(axum::body::Body::from("Internal Server Error")).unwrap()
                    }
                }
            ));

                let vm_arc = {
                    let vm_ref = ctx
                        .as_any()
                        .downcast_mut::<crate::machine::VM>()
                        .expect("Bukan VM");
                    std::sync::Arc::new(std::sync::Mutex::new(vm_ref.clone_vm()))
                };

                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                match tokio::net::TcpListener::bind(&addr).await {
                    Ok(listener) => {
                        println!("\x1b[32m🚀 Menjalankan Server Web di http://{}\x1b[0m", addr);
                        let _ = axum::serve(listener, app.with_state(vm_arc)).await;
                    },
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::AddrInUse {
                            let port = addr.split(':').next_back().unwrap_or("");
                            println!("\x1b[33m\n⚠️  PERINGATAN: Port {} sudah digunakan oleh program lain atau server RPL belum ditutup dengan benar!\x1b[0m", port);
                            println!("\x1b[33mSilakan tutup program yang berjalan di port tersebut (misal: tekan Ctrl+C pada terminal sebelumnya) atau gunakan port lain di server.rpl Anda.\n\x1b[0m");
                            std::process::exit(1);
                        } else {
                            println!("\x1b[31m\n❌ Gagal memulai server: {}\n\x1b[0m", e);
                            std::process::exit(1);
                        }
                    }
                }
            });

                Ok(Value::Kosong)
            },
        ),
    };
    let jalankan_idx = vm.heap.alloc(HeapData::FungsiBawaan(jalankan_func));
    web_map.insert("jalankan".to_string(), Value::FungsiBawaan(jalankan_idx));

    let web_idx = vm.heap.alloc(HeapData::Kamus(web_map));
    vm.set_global("web".to_string(), Value::Kamus(web_idx));
}

// Find a matching route, supporting :param dynamic segments
fn find_route(
    heap: &crate::heap::Heap,
    url: &str,
    method: &str,
) -> (Option<Value>, HashMap<String, String>) {
    // Try exact match first
    if let Some(method_map) = heap.web_routes.get(url)
        && let Some(func) = method_map.get(method)
    {
        return (Some(*func), HashMap::new());
    }

    // Try pattern match with :param
    let url_parts: Vec<&str> = url.trim_matches('/').split('/').collect();

    for (pattern, method_map) in &heap.web_routes {
        if !pattern.contains(':') {
            continue;
        }

        let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
        if pattern_parts.len() != url_parts.len() {
            continue;
        }

        let mut params = HashMap::new();
        let mut matched = true;

        for (pp, up) in pattern_parts.iter().zip(url_parts.iter()) {
            if let Some(stripped) = pp.strip_prefix(':') {
                params.insert(stripped.to_string(), up.to_string());
            } else if pp != up {
                matched = false;
                break;
            }
        }

        if matched && let Some(func) = method_map.get(method) {
            return (Some(*func), params);
        }
    }

    (None, HashMap::new())
}

async fn dev_ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(vm_state): axum::extract::State<
        std::sync::Arc<std::sync::Mutex<crate::machine::VM>>,
    >,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, vm_state))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    vm_state: std::sync::Arc<std::sync::Mutex<crate::machine::VM>>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let mut senders = crate::stdlib::dev_dashboard::get_ws_broadcast()
            .lock()
            .unwrap();
        senders.push(tx);
    }

    let snapshot = {
        let vm = vm_state.lock().unwrap();

        let mut routes_list = Vec::new();
        for (path, method_map) in &vm.heap.web_routes {
            for (method, val) in method_map {
                routes_list.push(serde_json::json!({
                    "method": method,
                    "path": path,
                    "handler": val.to_string(&vm.heap),
                    "file": "server.rpl",
                }));
            }
        }

        let mut sessions_list = Vec::new();
        if let Ok(sess_map) = vm.heap.web_state.sessions.lock() {
            for (sid, (exp, kamus_idx)) in &*sess_map {
                let dict = vm.heap.get_kamus(*kamus_idx);
                for (k, v) in dict {
                    sessions_list.push(serde_json::json!({
                        "id": sid,
                        "key": k,
                        "val": v.to_string(&vm.heap),
                        "expires": exp.map(|e| format!("{:?}", e.duration_since(std::time::Instant::now()))),
                    }));
                }
            }
        }

        let mut cache_list = Vec::new();
        if let Ok(cache) = vm.heap.web_cache.lock() {
            for (k, (ct, content)) in &cache.static_files {
                cache_list.push(serde_json::json!({
                    "key": k,
                    "content_type": ct,
                    "size": content.len(),
                }));
            }
        }

        let mut config_map = serde_json::Map::new();
        for (k, v) in std::env::vars() {
            if k.starts_with("RPL_") || k == "PORT" || k == "ENV" {
                config_map.insert(k, serde_json::Value::String(v));
            }
        }

        serde_json::json!({
            "event": "init",
            "data": {
                "project_name": std::env::var("RPL_PROJECT_NAME").unwrap_or_else(|_| "Proyek RPL".to_string()),
                "routes": routes_list,
                "sessions": sessions_list,
                "cache": cache_list,
                "config": config_map,
            }
        })
    };

    let (mut ws_tx, mut ws_rx) = socket.split();

    if ws_tx
        .send(axum::extract::ws::Message::Text(snapshot.to_string()))
        .await
        .is_err()
    {
        return;
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx
                .send(axum::extract::ws::Message::Text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let vm_state_metrics = vm_state.clone();
    let metrics_task = tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let ram_mb = {
                let vm = vm_state_metrics.lock().unwrap();
                vm.heap.allocated_count as f64 * 0.05
            };

            let uptime_secs = start_time.elapsed().as_secs();
            let h = uptime_secs / 3600;
            let m = (uptime_secs % 3600) / 60;
            let s = uptime_secs % 60;
            let uptime_str = format!("{:02}:{:02}:{:02}", h, m, s);

            let payload = serde_json::json!({
                "event": "metrics",
                "data": {
                    "uptime": uptime_str,
                    "ram": format!("{:.1} MB", ram_mb),
                    "cpu": 0,
                    "reqs_sec": 0.0,
                }
            })
            .to_string();

            crate::stdlib::dev_dashboard::broadcast_message(payload);
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let axum::extract::ws::Message::Text(text) = msg
            && let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text)
            && let Some(action) = cmd.get("action").and_then(|v| v.as_str())
        {
            match action {
                "clear_sessions" => {
                    if let Ok(mut sess_map) =
                        vm_state.lock().unwrap().heap.web_state.sessions.lock()
                    {
                        sess_map.clear();
                    }
                }
                "delete_session" => {
                    if let Some(data) = cmd.get("data")
                        && let (Some(sid), Some(key)) = (
                            data.get("session_id").and_then(|v| v.as_str()),
                            data.get("key").and_then(|v| v.as_str()),
                        )
                        && let Ok(sess_map) =
                            vm_state.lock().unwrap().heap.web_state.sessions.lock()
                        && let Some((_, kamus_idx)) = sess_map.get(sid)
                    {
                        let mut vm = vm_state.lock().unwrap();
                        vm.heap.get_kamus_mut(*kamus_idx).remove(key);
                    }
                }
                "clear_cache" => {
                    if let Ok(mut cache) = vm_state.lock().unwrap().heap.web_cache.lock() {
                        cache.static_files.clear();
                        cache.templates_code.clear();
                    }
                }
                "delete_cache" => {
                    if let Some(data) = cmd.get("data")
                        && let Some(key) = data.get("key").and_then(|v| v.as_str())
                        && let Ok(mut cache) = vm_state.lock().unwrap().heap.web_cache.lock()
                    {
                        cache.static_files.remove(key);
                        cache.templates_code.remove(key);
                    }
                }
                "replay_request" => {
                    if let Some(data) = cmd.get("data") {
                        let method = data
                            .get("method")
                            .and_then(|v| v.as_str())
                            .unwrap_or("GET")
                            .to_string();
                        let url = data
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("/")
                            .to_string();
                        let body = data
                            .get("body")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let port = 3001;
                        let client_url = format!("http://localhost:{}{}", port, url);
                        tokio::spawn(async move {
                            let method_str = method.as_str();
                            if method_str == "POST" {
                                let _ = ureq::post(&client_url).send(&body);
                            } else if method_str == "PUT" {
                                let _ = ureq::put(&client_url).send(&body);
                            } else if method_str == "DELETE" {
                                let _ = ureq::delete(&client_url).call();
                            } else {
                                let _ = ureq::get(&client_url).call();
                            }
                        });
                    }
                }
                _ => {}
            }
        }
    }

    send_task.abort();
    metrics_task.abort();
}

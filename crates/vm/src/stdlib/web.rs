use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use crate::heap::HeapData;
use std::collections::HashMap;
use std::time::Instant;
use std::io::{Read, Write};
use flate2::write::GzEncoder;
use flate2::Compression;

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

    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'web.get' membutuhkan 2 argumen (path, handler)".to_string());
            }
            let path = args[0].to_string(ctx.get_heap_mut());
            let func_idx = match args[1] {
                Value::Fungsi(idx) => idx,
                _ => return Err("Argumen kedua 'web.get' harus berupa fungsi".to_string()),
            };
            ctx.get_heap_mut().web_routes.insert(path, func_idx);
            Ok(Value::Kosong)
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    web_map.insert("get".to_string(), Value::FungsiBawaan(get_idx));
    
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
            println!("🚀 Menjalankan Server Web Native RPL di http://{}", addr);
            
            let kompresi_aktif = ctx.get_heap_mut().web_config.kompresi;
            let rate_limit = ctx.get_heap_mut().web_config.rate_limit;
            let proxies = ctx.get_heap_mut().web_config.proxies.clone(); // Clone to use in loop
            
            let server = tiny_http::Server::http(&addr)
                .map_err(|e| format!("Gagal menjalankan server: {}", e))?;
                
            let mut rate_limits: HashMap<String, (Instant, u32)> = HashMap::new();
                
            'req_loop: for mut request in server.incoming_requests() {
                let url = request.url().to_string();
                
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

                        let method = request.method().as_str();
                        let res = match method {
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
                
                // 3. Normal Routing
                let route_opt = ctx.get_heap_mut().web_routes.get(&url).copied();
                match route_opt {
                    Some(func_idx) => {
                        let req_kamus_idx = {
                            let mut req_map = HashMap::new();
                            let url_str = ctx.get_heap_mut().alloc(HeapData::String(url));
                            req_map.insert("url".to_string(), Value::String(url_str));
                            
                            let method_str = ctx.get_heap_mut().alloc(HeapData::String(request.method().as_str().to_string()));
                            req_map.insert("metode".to_string(), Value::String(method_str));
                            
                            let mut body = String::new();
                            let _ = request.as_reader().read_to_string(&mut body);
                            let body_str = ctx.get_heap_mut().alloc(HeapData::String(body));
                            req_map.insert("tubuh".to_string(), Value::String(body_str));
                            
                            ctx.get_heap_mut().alloc(HeapData::Kamus(req_map))
                        };
                        
                        let req_val = Value::Kamus(req_kamus_idx);
                        let hasil = ctx.execute_function(func_idx, vec![req_val]);
                        
                        match hasil {
                            Ok(val) => {
                                let val_string = val.to_string(ctx.get_heap_mut());
                                let is_json = matches!(val, Value::Kamus(_) | Value::Array(_));
                                let content_type = if is_json { "application/json" } else { "text/html" };
                                
                                let mut accept_encoding = String::new();
                                for h in request.headers() {
                                    if h.field.equiv("Accept-Encoding") {
                                        accept_encoding = h.value.as_str().to_lowercase();
                                        break;
                                    }
                                }
                                
                                if kompresi_aktif && accept_encoding.contains("br") {
                                    let mut compressed = Vec::new();
                                    let mut cursor = std::io::Cursor::new(val_string.as_bytes());
                                    let _ = brotli::CompressorReader::new(&mut cursor, 4096, 11, 22).read_to_end(&mut compressed);
                                    let mut response = tiny_http::Response::from_data(compressed);
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Encoding"[..], &b"br"[..]).unwrap());
                                    let _ = request.respond(response);
                                } else if kompresi_aktif && accept_encoding.contains("gzip") {
                                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                                    let _ = encoder.write_all(val_string.as_bytes());
                                    let compressed = encoder.finish().unwrap_or_default();
                                    let mut response = tiny_http::Response::from_data(compressed);
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Encoding"[..], &b"gzip"[..]).unwrap());
                                    let _ = request.respond(response);
                                } else {
                                    let mut response = tiny_http::Response::from_string(val_string);
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
                                    let _ = request.respond(response);
                                }
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

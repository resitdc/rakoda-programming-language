use crate::machine::VM;
use crate::value::{Value, FungsiBawaanVM};
use crate::heap::HeapData;
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
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
            
            // Store route in heap
            ctx.get_heap_mut().web_routes.insert(path, func_idx);
            
            Ok(Value::Kosong)
        },
    };
    
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
            
            let server = tiny_http::Server::http(&addr)
                .map_err(|e| format!("Gagal menjalankan server: {}", e))?;
                
            for mut request in server.incoming_requests() {
                let url = request.url().to_string();
                
                // Cari route
                let route_opt = ctx.get_heap_mut().web_routes.get(&url).copied();
                
                match route_opt {
                    Some(func_idx) => {
                        // Siapkan argumen request
                        let req_kamus_idx = {
                            let mut req_map = HashMap::new();
                            let url_str = ctx.get_heap_mut().alloc(HeapData::String(url));
                            req_map.insert("url".to_string(), Value::String(url_str));
                            
                            let method_str = ctx.get_heap_mut().alloc(HeapData::String(request.method().as_str().to_string()));
                            req_map.insert("metode".to_string(), Value::String(method_str));
                            
                            // Baca body
                            let mut body = String::new();
                            let _ = request.as_reader().read_to_string(&mut body);
                            let body_str = ctx.get_heap_mut().alloc(HeapData::String(body));
                            req_map.insert("tubuh".to_string(), Value::String(body_str));
                            
                            ctx.get_heap_mut().alloc(HeapData::Kamus(req_map))
                        };
                        
                        let req_val = Value::Kamus(req_kamus_idx);
                        
                        // Eksekusi fungsi pengguna
                        let hasil = ctx.execute_function(func_idx, vec![req_val]);
                        
                        match hasil {
                            Ok(val) => {
                                // Jika hasil Kamus/Array, ubah ke JSON
                                if matches!(val, Value::Kamus(_) | Value::Array(_)) {
                                    // Convert to JSON (simple hack: use json::stringify if available, or just fallback)
                                    // Let's manually construct JSON for now, or just use string
                                    let mut response = tiny_http::Response::from_string(val.to_string(ctx.get_heap_mut()));
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                                    let _ = request.respond(response);
                                } else {
                                    let mut response = tiny_http::Response::from_string(val.to_string(ctx.get_heap_mut()));
                                    response.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
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
                        let resp = tiny_http::Response::from_string("Not Found")
                            .with_status_code(404);
                        let _ = request.respond(resp);
                    }
                }
            }
            
            Ok(Value::Kosong)
        },
    };
    
    let mut web_map = HashMap::new();
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    web_map.insert("get".to_string(), Value::FungsiBawaan(get_idx));
    
    let jalankan_idx = vm.heap.alloc(HeapData::FungsiBawaan(jalankan_func));
    web_map.insert("jalankan".to_string(), Value::FungsiBawaan(jalankan_idx));
    
    let web_idx = vm.heap.alloc(HeapData::Kamus(web_map));
    vm.set_global("web".to_string(), Value::Kamus(web_idx));
}

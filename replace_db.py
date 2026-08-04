import re

with open('crates/vm/src/stdlib/db.rs', 'r') as f:
    content = f.read()

# Replace eksekusi_func
eksekusi_replacement = """    let eksekusi_func = FungsiBawaanVM {
        nama: "eksekusi".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                let pool = match &vm.get_heap_mut().db_pool {
                    Some(p) => p.clone(),
                    None => {
                        return Err(
                            "Koneksi database belum dibuka. Panggil db.hubungkan() terlebih dahulu"
                                .to_string(),
                        );
                    }
                };
                eksekusi_helper(vm, &pool, &args)
            },
        ),
    };"""
content = re.sub(r'    let eksekusi_func = FungsiBawaanVM \{.*?^\s*\}\,\n\s*\)\,\n\s*\};', eksekusi_replacement, content, flags=re.MULTILINE | re.DOTALL)

# Replace kueri_func
kueri_replacement = """    let kueri_func = FungsiBawaanVM {
        nama: "kueri".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                let pool = match &vm.get_heap_mut().db_pool {
                    Some(p) => p.clone(),
                    None => {
                        return Err(
                            "Koneksi database belum dibuka. Panggil db.hubungkan() terlebih dahulu"
                                .to_string(),
                        );
                    }
                };
                kueri_helper(vm, &pool, &args)
            },
        ),
    };"""
content = re.sub(r'    let kueri_func = FungsiBawaanVM \{.*?^\s*\}\,\n\s*\)\,\n\s*\};', kueri_replacement, content, flags=re.MULTILINE | re.DOTALL)

# Update hubungkan_func
hubungkan_replacement = """    let hubungkan_func = FungsiBawaanVM {
        nama: "hubungkan".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.is_empty() {
                    return Err("db.hubungkan membutuhkan 1 argumen: URL koneksi".to_string());
                }

                let url_str = match args[0] {
                    Value::String(idx) => vm.get_heap_mut().get_string(idx).clone(),
                    Value::Kamus(idx) => {
                        let kamus = vm.get_heap_mut().get_kamus(idx).clone();

                        let mut provider = "sqlite".to_string();
                        let mut host = "localhost".to_string();
                        let mut nama = "test.db".to_string();
                        let mut username = "root".to_string();
                        let mut password = "".to_string();
                        let mut port = "".to_string();

                        for (k, v) in kamus {
                            let val_str = match v {
                                Value::String(s_idx) => vm.get_heap_mut().get_string(s_idx).clone(),
                                Value::Angka(n) => (n as i64).to_string(),
                                _ => continue,
                            };
                            match k.as_str() {
                                "provider" => provider = val_str,
                                "host" => host = val_str,
                                "nama" => nama = val_str,
                                "username" => username = val_str,
                                "password" => password = val_str,
                                "port" => port = format!(":{}", val_str),
                                _ => {}
                            }
                        }

                        if provider == "sqlite" {
                            format!("sqlite://{}", nama)
                        } else {
                            let auth = if password.is_empty() {
                                username
                            } else {
                                format!("{}:{}", username, password)
                            };
                            format!("{}://{}@{}{}/{}", provider, auth, host, port, nama)
                        }
                    }
                    _ => {
                        return Err(
                            "Koneksi harus berupa teks URL atau kamus konfigurasi".to_string()
                        );
                    }
                };

                let pool = if url_str.starts_with("sqlite://") {
                    let raw_path = url_str.trim_start_matches("sqlite://");
                    let path = if std::path::Path::new(raw_path).is_relative() {
                        if let Some(root) = &vm.get_heap_mut().project_root {
                            root.join(raw_path).to_string_lossy().to_string()
                        } else {
                            raw_path.to_string()
                        }
                    } else {
                        raw_path.to_string()
                    };
                    DbPool::new_sqlite_pool(&path, 5)?
                } else if url_str.starts_with("mysql://") {
                    let opts = mysql::Opts::from_url(&url_str)
                        .map_err(|e| format!("URL MySQL tidak valid: {}", e))?;
                    let builder = mysql::OptsBuilder::from_opts(opts);
                    let pool = mysql::Pool::new(builder)
                        .map_err(|e| format!("Gagal membuat pool MySQL: {}", e))?;
                    DbPool::Mysql(pool)
                } else if url_str.starts_with("postgres://") {
                    let manager = r2d2_postgres::PostgresConnectionManager::new(
                        url_str.parse().map_err(|e| format!("URL Postgres salah: {}", e))?,
                        r2d2_postgres::postgres::NoTls,
                    );
                    let pool = r2d2::Pool::builder()
                        .max_size(10)
                        .build(manager)
                        .map_err(|e| format!("Gagal membuat pool Postgres: {}", e))?;
                    DbPool::Postgres(pool)
                } else {
                    return Err(format!("Protokol tidak didukung: {}", url_str));
                };

                vm.get_heap_mut().db_pool = Some(pool.clone());
                
                let pool_idx = vm.get_heap_mut().alloc(HeapData::DbPool(pool));
                
                let mut dict = HashMap::new();
                
                let eksekusi_func = FungsiBawaanVM {
                    nama: "eksekusi".to_string(),
                    func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                        let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                            HeapData::DbPool(p) => p.clone(),
                            _ => return Err("Koneksi tidak valid".to_string()),
                        };
                        eksekusi_helper(vm, &pool, &args)
                    }),
                };
                let eksekusi_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(eksekusi_func));
                dict.insert("eksekusi".to_string(), Value::FungsiBawaan(eksekusi_idx));
                
                let kueri_func = FungsiBawaanVM {
                    nama: "kueri".to_string(),
                    func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                        let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                            HeapData::DbPool(p) => p.clone(),
                            _ => return Err("Koneksi tidak valid".to_string()),
                        };
                        kueri_helper(vm, &pool, &args)
                    }),
                };
                let kueri_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(kueri_func));
                dict.insert("kueri".to_string(), Value::FungsiBawaan(kueri_idx));

                let dict_idx = vm.get_heap_mut().alloc(HeapData::Kamus(dict));
                Ok(Value::Kamus(dict_idx))
            },
        ),
    };"""
content = re.sub(r'    let hubungkan_func = FungsiBawaanVM \{.*?^\s*\}\,\n\s*\)\,\n\s*\};', hubungkan_replacement, content, flags=re.MULTILINE | re.DOTALL)

with open('crates/vm/src/stdlib/db.rs', 'w') as f:
    f.write(content)

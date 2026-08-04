use crate::heap::{DatabaseConnection, DbPool, DbQueryState, HeapData};
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let hubungkan_func = FungsiBawaanVM {
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
                dict.insert("query".to_string(), Value::FungsiBawaan(eksekusi_idx)); // Alias

                // --- QUERY BUILDER (Knex Style) ---
                let table_func = FungsiBawaanVM {
                    nama: "table".to_string(),
                    func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                        if args.is_empty() {
                            return Err("table membutuhkan nama tabel".to_string());
                        }
                        let tabel = match &args[0] {
                            Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                            _ => return Err("Nama tabel harus teks".to_string()),
                        };
                        
                        let query_state = DbQueryState {
                            tabel,
                            schema: None,
                            kondisi: vec![],
                            limit_val: None,
                            offset_val: None,
                        };
                        let state_idx = vm.get_heap_mut().alloc(HeapData::QueryState(query_state));
                        
                        let mut qb_dict = std::collections::HashMap::new();
                        let qb_dict_idx = vm.get_heap_mut().alloc(HeapData::Kamus(qb_dict.clone()));

                        let where_func = FungsiBawaanVM {
                            nama: "where".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.len() < 3 { return Err("where butuh 3 argumen".to_string()); }
                                let kolom = match &args[0] {
                                    Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                                    _ => return Err("Kolom harus teks".to_string()),
                                };
                                let operator = match &args[1] {
                                    Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                                    _ => return Err("Operator harus teks".to_string()),
                                };
                                let nilai = args[2];
                                
                                if let HeapData::QueryState(state) = &mut vm.get_heap_mut().objects[state_idx].data {
                                    state.kondisi.push((kolom, operator, nilai));
                                }
                                Ok(Value::Kamus(qb_dict_idx))
                            }),
                        };
                        let where_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(where_func));

                        let select_func = FungsiBawaanVM {
                            nama: "select".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, _args: Vec<Value>| {
                                let sql = {
                                    let state = match &vm.get_heap_mut().objects[state_idx].data {
                                        HeapData::QueryState(s) => s.clone(),
                                        _ => return Err("Query state tidak valid".to_string()),
                                    };
                                                                        let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("SELECT * FROM {}", table_name);
                                    if !state.kondisi.is_empty() {
                                        query.push_str(" WHERE ");
                                        let mut conds = Vec::new();
                                        for (k, o, v) in state.kondisi {
                                            let val_str = match v {
                                                Value::Angka(n) => n.to_string(),
                                                Value::String(idx) => {
                                                    let s = vm.get_heap_mut().get_string(idx);
                                                    format!("'{}'", s.replace('\'', "''"))
                                                }
                                                Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
                                                Value::Kosong => "NULL".to_string(),
                                                _ => "''".to_string(),
                                            };
                                            conds.push(format!("{} {} {}", k, o, val_str));
                                        }
                                        query.push_str(&conds.join(" AND "));
                                    }
                                    if let Some(l) = state.limit_val {
                                        query.push_str(&format!(" LIMIT {}", l));
                                    }
                                    if let Some(o) = state.offset_val {
                                        query.push_str(&format!(" OFFSET {}", o));
                                    }
                                    query
                                };
                                let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                                    HeapData::DbPool(p) => p.clone(),
                                    _ => return Err("Koneksi tidak valid".to_string()),
                                };
                                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));
                                kueri_helper(vm, &pool, &[Value::String(sql_idx)])
                            }),
                        };
                        let select_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(select_func));

                        let insert_func = FungsiBawaanVM {
                            nama: "insert".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.is_empty() { return Err("insert butuh argumen kamus atau array kamus".to_string()); }
                                
                                let mut is_multi = false;
                                let mut dicts = Vec::new();
                                
                                match &args[0] {
                                    Value::Kamus(idx) => {
                                        dicts.push(vm.get_heap_mut().get_kamus(*idx).clone());
                                    }
                                    Value::Array(idx) => {
                                        is_multi = true;
                                        let arr = vm.get_heap_mut().get_array(*idx).clone();
                                        for v in arr {
                                            if let Value::Kamus(d_idx) = v {
                                                dicts.push(vm.get_heap_mut().get_kamus(d_idx).clone());
                                            } else {
                                                return Err("Array harus berisi kamus".to_string());
                                            }
                                        }
                                    }
                                    _ => return Err("insert butuh kamus atau array kamus".to_string()),
                                };
                                
                                if dicts.is_empty() {
                                    return Ok(Value::Angka(0.0));
                                }

                                let sql = {
                                    let state = match &vm.get_heap_mut().objects[state_idx].data {
                                        HeapData::QueryState(s) => s.clone(),
                                        _ => return Err("Query state tidak valid".to_string()),
                                    };
                                    let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    
                                    // Use columns from the first object
                                    let mut cols = Vec::new();
                                    for k in dicts[0].keys() {
                                        cols.push(k.clone());
                                    }
                                    
                                    let mut all_vals = Vec::new();
                                    
                                    for d in dicts {
                                        let mut vals = Vec::new();
                                        for c in &cols {
                                            let v = d.get(c).unwrap_or(&Value::Kosong);
                                            let val_str = match v {
                                                Value::Angka(n) => n.to_string(),
                                                Value::String(idx) => {
                                                    let s = vm.get_heap_mut().get_string(*idx);
                                                    format!("'{}'", s.replace('\'', "''"))
                                                }
                                                Value::Boolean(b) => if *b { "1".to_string() } else { "0".to_string() },
                                                Value::Kosong => "NULL".to_string(),
                                                _ => "''".to_string(),
                                            };
                                            vals.push(val_str);
                                        }
                                        all_vals.push(format!("({})", vals.join(", ")));
                                    }
                                    format!("INSERT INTO {} ({}) VALUES {}", table_name, cols.join(", "), all_vals.join(", "))
                                };
                                let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                                    HeapData::DbPool(p) => p.clone(),
                                    _ => return Err("Koneksi tidak valid".to_string()),
                                };
                                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));
                                eksekusi_helper(vm, &pool, &[Value::String(sql_idx)])
                            }),
                        };
                        let insert_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(insert_func));
                        
                        let delete_func = FungsiBawaanVM {
                            nama: "delete".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, _args: Vec<Value>| {
                                let sql = {
                                    let state = match &vm.get_heap_mut().objects[state_idx].data {
                                        HeapData::QueryState(s) => s.clone(),
                                        _ => return Err("Query state tidak valid".to_string()),
                                    };
                                                                        let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("DELETE FROM {}", table_name);
                                    if !state.kondisi.is_empty() {
                                        query.push_str(" WHERE ");
                                        let mut conds = Vec::new();
                                        for (k, o, v) in state.kondisi {
                                            let val_str = match v {
                                                Value::Angka(n) => n.to_string(),
                                                Value::String(idx) => {
                                                    let s = vm.get_heap_mut().get_string(idx);
                                                    format!("'{}'", s.replace('\'', "''"))
                                                }
                                                Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
                                                Value::Kosong => "NULL".to_string(),
                                                _ => "''".to_string(),
                                            };
                                            conds.push(format!("{} {} {}", k, o, val_str));
                                        }
                                        query.push_str(&conds.join(" AND "));
                                    }
                                    query
                                };
                                let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                                    HeapData::DbPool(p) => p.clone(),
                                    _ => return Err("Koneksi tidak valid".to_string()),
                                };
                                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));
                                eksekusi_helper(vm, &pool, &[Value::String(sql_idx)])
                            }),
                        };
                        let delete_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(delete_func));
                        
                        let update_func = FungsiBawaanVM {
                            nama: "update".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.is_empty() { return Err("update butuh argumen kamus".to_string()); }
                                let dict_idx = match args[0] {
                                    Value::Kamus(idx) => idx,
                                    _ => return Err("update butuh kamus".to_string()),
                                };
                                let sql = {
                                    let data = vm.get_heap_mut().get_kamus(dict_idx).clone();
                                    let state = match &vm.get_heap_mut().objects[state_idx].data {
                                        HeapData::QueryState(s) => s.clone(),
                                        _ => return Err("Query state tidak valid".to_string()),
                                    };
                                    let mut sets = Vec::new();
                                    for (k, v) in data {
                                        let val_str = match v {
                                            Value::Angka(n) => n.to_string(),
                                            Value::String(idx) => {
                                                let s = vm.get_heap_mut().get_string(idx);
                                                format!("'{}'", s.replace('\'', "''"))
                                            }
                                            Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
                                            Value::Kosong => "NULL".to_string(),
                                            _ => "''".to_string(),
                                        };
                                        sets.push(format!("{} = {}", k, val_str));
                                    }
                                                                        let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("UPDATE {} SET {}", table_name, sets.join(", "));
                                    if !state.kondisi.is_empty() {
                                        query.push_str(" WHERE ");
                                        let mut conds = Vec::new();
                                        for (k, o, v) in state.kondisi {
                                            let val_str = match v {
                                                Value::Angka(n) => n.to_string(),
                                                Value::String(idx) => {
                                                    let s = vm.get_heap_mut().get_string(idx);
                                                    format!("'{}'", s.replace('\'', "''"))
                                                }
                                                Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
                                                Value::Kosong => "NULL".to_string(),
                                                _ => "''".to_string(),
                                            };
                                            conds.push(format!("{} {} {}", k, o, val_str));
                                        }
                                        query.push_str(&conds.join(" AND "));
                                    }
                                    query
                                };
                                let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                                    HeapData::DbPool(p) => p.clone(),
                                    _ => return Err("Koneksi tidak valid".to_string()),
                                };
                                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));
                                eksekusi_helper(vm, &pool, &[Value::String(sql_idx)])
                            }),
                        };
                        let update_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(update_func));


                        let with_schema_func = FungsiBawaanVM {
                            nama: "withSchema".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.is_empty() { return Err("withSchema butuh nama schema".to_string()); }
                                let schema = match &args[0] {
                                    Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                                    _ => return Err("Schema harus teks".to_string()),
                                };
                                if let HeapData::QueryState(state) = &mut vm.get_heap_mut().objects[state_idx].data {
                                    state.schema = Some(schema);
                                }
                                Ok(Value::Kamus(qb_dict_idx))
                            }),
                        };
                        let with_schema_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(with_schema_func));

                        let limit_func = FungsiBawaanVM {
                            nama: "limit".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.is_empty() { return Err("limit butuh angka".to_string()); }
                                let limit_val = match args[0] {
                                    Value::Angka(n) => n as usize,
                                    _ => return Err("Limit harus angka".to_string()),
                                };
                                if let HeapData::QueryState(state) = &mut vm.get_heap_mut().objects[state_idx].data {
                                    state.limit_val = Some(limit_val);
                                }
                                Ok(Value::Kamus(qb_dict_idx))
                            }),
                        };
                        let limit_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(limit_func));

                        let offset_func = FungsiBawaanVM {
                            nama: "offset".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.is_empty() { return Err("offset butuh angka".to_string()); }
                                let offset_val = match args[0] {
                                    Value::Angka(n) => n as usize,
                                    _ => return Err("Offset harus angka".to_string()),
                                };
                                if let HeapData::QueryState(state) = &mut vm.get_heap_mut().objects[state_idx].data {
                                    state.offset_val = Some(offset_val);
                                }
                                Ok(Value::Kamus(qb_dict_idx))
                            }),
                        };
                        let offset_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(offset_func));

                        let first_func = FungsiBawaanVM {
                            nama: "first".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, _args: Vec<Value>| {
                                if let HeapData::QueryState(state) = &mut vm.get_heap_mut().objects[state_idx].data {
                                    state.limit_val = Some(1);
                                }
                                // Execute select
                                let select_val = {
                                    if let HeapData::Kamus(d) = &vm.get_heap_mut().objects[qb_dict_idx].data {
                                        d.get("select").cloned().unwrap()
                                    } else {
                                        return Err("Gagal mengambil fungsi select".to_string());
                                    }
                                };
                                let res = vm.execute_function(select_val, vec![])?;
                                if let Value::Array(arr_idx) = res {
                                    let arr = vm.get_heap_mut().get_array(arr_idx).clone();
                                    if arr.is_empty() {
                                        Ok(Value::Kosong)
                                    } else {
                                        Ok(arr[0].clone())
                                    }
                                } else {
                                    Ok(res)
                                }
                            }),
                        };
                        let first_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(first_func));

                        let upsert_func = FungsiBawaanVM {
                            nama: "upsert".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.len() < 2 { return Err("upsert butuh data dan array conflict columns".to_string()); }
                                let dict_idx = match args[0] {
                                    Value::Kamus(idx) => idx,
                                    _ => return Err("argumen pertama upsert butuh kamus".to_string()),
                                };
                                let conflict_cols = match args[1] {
                                    Value::Array(idx) => {
                                        let arr = vm.get_heap_mut().get_array(idx).clone();
                                        let mut cols = Vec::new();
                                        for v in arr {
                                            if let Value::String(s_idx) = v {
                                                cols.push(vm.get_heap_mut().get_string(s_idx).clone());
                                            }
                                        }
                                        cols
                                    }
                                    _ => return Err("argumen kedua upsert butuh array of string".to_string()),
                                };

                                let pool = match &vm.get_heap_mut().objects[pool_idx].data {
                                    HeapData::DbPool(p) => p.clone(),
                                    _ => return Err("Koneksi tidak valid".to_string()),
                                };
                                let provider = pool.provider_name().to_string();

                                let sql = {
                                    let data = vm.get_heap_mut().get_kamus(dict_idx).clone();
                                    let state = match &vm.get_heap_mut().objects[state_idx].data {
                                        HeapData::QueryState(s) => s.clone(),
                                        _ => return Err("Query state tidak valid".to_string()),
                                    };
                                    let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    
                                    let mut cols = Vec::new();
                                    let mut vals = Vec::new();
                                    let mut set_clauses = Vec::new();

                                    for (k, v) in data {
                                        cols.push(k.clone());
                                        let val_str = match v {
                                            Value::Angka(n) => n.to_string(),
                                            Value::String(idx) => {
                                                let s = vm.get_heap_mut().get_string(idx);
                                                format!("'{}'", s.replace('\'', "''"))
                                            }
                                            Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
                                            Value::Kosong => "NULL".to_string(),
                                            _ => "''".to_string(),
                                        };
                                        vals.push(val_str.clone());
                                        
                                        // Build ON DUPLICATE KEY UPDATE / ON CONFLICT DO UPDATE SET
                                        if provider == "mysql" {
                                            set_clauses.push(format!("{} = VALUES({})", k, k));
                                        } else {
                                            set_clauses.push(format!("{} = EXCLUDED.{}", k, k));
                                        }
                                    }
                                    
                                    if provider == "mysql" {
                                        format!("INSERT INTO {} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}", 
                                            table_name, cols.join(", "), vals.join(", "), set_clauses.join(", "))
                                    } else {
                                        format!("INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {}", 
                                            table_name, cols.join(", "), vals.join(", "), conflict_cols.join(", "), set_clauses.join(", "))
                                    }
                                };
                                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));
                                eksekusi_helper(vm, &pool, &[Value::String(sql_idx)])
                            }),
                        };
                        let upsert_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(upsert_func));

                        if let HeapData::Kamus(d) = &mut vm.get_heap_mut().objects[qb_dict_idx].data {
                            d.insert("where".to_string(), Value::FungsiBawaan(where_idx));
                            d.insert("select".to_string(), Value::FungsiBawaan(select_idx));
                            d.insert("insert".to_string(), Value::FungsiBawaan(insert_idx));
                            d.insert("update".to_string(), Value::FungsiBawaan(update_idx));
                            d.insert("delete".to_string(), Value::FungsiBawaan(delete_idx));
                            d.insert("withSchema".to_string(), Value::FungsiBawaan(with_schema_idx));
                            d.insert("limit".to_string(), Value::FungsiBawaan(limit_idx));
                            d.insert("offset".to_string(), Value::FungsiBawaan(offset_idx));
                            d.insert("first".to_string(), Value::FungsiBawaan(first_idx));
                            d.insert("upsert".to_string(), Value::FungsiBawaan(upsert_idx));
                        }


                        Ok(Value::Kamus(qb_dict_idx))
                    }),
                };
                let table_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(table_func));
                dict.insert("table".to_string(), Value::FungsiBawaan(table_idx));


                let dict_idx = vm.get_heap_mut().alloc(HeapData::Kamus(dict));
                Ok(Value::Kamus(dict_idx))
            },
        ),
    };

    let eksekusi_func = FungsiBawaanVM {
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
    };

    let kueri_func = FungsiBawaanVM {
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
    };

    let hubungkan_idx = vm.heap.alloc(HeapData::FungsiBawaan(hubungkan_func));
    let eksekusi_idx = vm.heap.alloc(HeapData::FungsiBawaan(eksekusi_func));
    let kueri_idx = vm.heap.alloc(HeapData::FungsiBawaan(kueri_func));

    module_dict.insert("hubungkan".to_string(), Value::FungsiBawaan(hubungkan_idx));
    module_dict.insert("eksekusi".to_string(), Value::FungsiBawaan(eksekusi_idx));
    module_dict.insert("kueri".to_string(), Value::FungsiBawaan(kueri_idx));
    module_dict.insert("query".to_string(), Value::FungsiBawaan(eksekusi_idx));

    // --- ORM (Objection Style) ---
    let model_func = FungsiBawaanVM {
        nama: "Model".to_string(),
        func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() { return Err("db.Model membutuhkan kamus konfigurasi".to_string()); }
            let config_idx = match args[0] {
                Value::Kamus(idx) => idx,
                _ => return Err("db.Model membutuhkan kamus konfigurasi".to_string()),
            };
            
            let mut model_dict = std::collections::HashMap::new();
            
            let query_func = FungsiBawaanVM {
                nama: "query".to_string(),
                func: std::sync::Arc::new(move |vm: &mut dyn VmContext, _args: Vec<Value>| {
                    let config_inner = vm.get_heap_mut().get_kamus(config_idx).clone();
                    let table_name_val = config_inner.get("tableName").cloned().ok_or_else(|| "tableName tidak ditemukan di config".to_string())?;
                    let conn_val = config_inner.get("connection").cloned().ok_or_else(|| "connection tidak ditemukan di config".to_string())?;
                    
                    let method_val = match conn_val {
                        Value::Kamus(idx) => {
                            let dict = vm.get_heap_mut().get_kamus(idx).clone();
                            dict.get("table").cloned().ok_or_else(|| "Koneksi tidak memiliki metode table".to_string())?
                        }
                        _ => return Err("connection harus berupa kamus koneksi".to_string()),
                    };
                    vm.execute_function(method_val, vec![table_name_val])
                })
            };
            let query_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(query_func));
            model_dict.insert("query".to_string(), Value::FungsiBawaan(query_idx));
            
            let model_idx = vm.get_heap_mut().alloc(HeapData::Kamus(model_dict));
            Ok(Value::Kamus(model_idx))
        })
    };
    let model_idx = vm.heap.alloc(HeapData::FungsiBawaan(model_func));
    module_dict.insert("Model".to_string(), Value::FungsiBawaan(model_idx));


    let tabel_func = FungsiBawaanVM {
        nama: "tabel".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.is_empty() {
                    return Err("db.tabel membutuhkan 1 argumen: nama tabel".to_string());
                }

                let nama = match &args[0] {
                    Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                    _ => return Err("Nama tabel harus berupa teks".to_string()),
                };

                let heap = vm.get_heap_mut();
                heap.db_query_state.tabel = nama;
                heap.db_query_state.kondisi.clear();

                let mod_idx = heap.db_module_idx.unwrap();
                Ok(Value::Modul(mod_idx))
            },
        ),
    };

    let dimana_func = FungsiBawaanVM {
        nama: "dimana".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.len() < 3 {
                    return Err(
                        "db.dimana membutuhkan 3 argumen: kolom, operator, nilai".to_string()
                    );
                }

                let kolom = match &args[0] {
                    Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                    _ => return Err("Kolom harus berupa teks".to_string()),
                };

                let operator = match &args[1] {
                    Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                    _ => return Err("Operator harus berupa teks".to_string()),
                };

                let nilai = args[2];

                let heap = vm.get_heap_mut();
                heap.db_query_state.kondisi.push((kolom, operator, nilai));

                let mod_idx = heap.db_module_idx.unwrap();
                Ok(Value::Modul(mod_idx))
            },
        ),
    };

    let ambil_func = FungsiBawaanVM {
        nama: "ambil".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, _args: Vec<Value>| -> Result<Value, String> {
                let sql = {
                    let state = vm.get_heap_mut().db_query_state.clone();
                    if state.tabel.is_empty() {
                        return Err("Panggil db.tabel() terlebih dahulu".to_string());
                    }

                                                        let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("SELECT * FROM {}", table_name);

                    if !state.kondisi.is_empty() {
                        query.push_str(" WHERE ");
                        let mut conds = Vec::new();
                        for (k, o, v) in state.kondisi {
                            let val_str = match v {
                                Value::Angka(n) => n.to_string(),
                                Value::String(idx) => {
                                    let s = vm.get_heap_mut().get_string(idx);
                                    format!("'{}'", s.replace('\'', "''"))
                                }
                                Value::Boolean(b) => {
                                    if b {
                                        "1".to_string()
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                Value::Kosong => "NULL".to_string(),
                                _ => "''".to_string(),
                            };
                            conds.push(format!("{} {} {}", k, o, val_str));
                        }
                        query.push_str(&conds.join(" AND "));
                    }
                    query
                };

                // Reset state
                vm.get_heap_mut().db_query_state.tabel.clear();
                vm.get_heap_mut().db_query_state.kondisi.clear();

                // Allocate sql string into heap and call db_kueri
                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));

                // We need to call kueri function. It is inside the module.
                let kueri_val = {
                    let mod_idx = vm.get_heap_mut().db_module_idx.unwrap();
                    let dict = vm.get_heap_mut().get_modul(mod_idx);
                    dict.get("kueri").cloned().unwrap()
                };

                vm.execute_function(kueri_val, vec![Value::String(sql_idx)])
            },
        ),
    };

    let simpan_func = FungsiBawaanVM {
        nama: "simpan".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.is_empty() {
                    return Err("db.simpan membutuhkan 1 argumen: data kamus".to_string());
                }

                let sql = {
                    let state = vm.get_heap_mut().db_query_state.clone();
                    if state.tabel.is_empty() {
                        return Err("Panggil db.tabel() terlebih dahulu".to_string());
                    }

                    let kamus_idx = match &args[0] {
                        Value::Kamus(idx) => *idx,
                        _ => return Err("Data harus berupa Kamus".to_string()),
                    };

                    let kamus = vm.get_heap_mut().get_kamus(kamus_idx).clone();
                    if kamus.is_empty() {
                        return Err("Data kamus kosong".to_string());
                    }

                    let mut cols = Vec::new();
                    let mut vals = Vec::new();

                    for (k, v) in kamus {
                        cols.push(k);
                        let val_str = match v {
                            Value::Angka(n) => n.to_string(),
                            Value::String(idx) => {
                                let s = vm.get_heap_mut().get_string(idx);
                                format!("'{}'", s.replace('\'', "''"))
                            }
                            Value::Boolean(b) => {
                                if b {
                                    "1".to_string()
                                } else {
                                    "0".to_string()
                                }
                            }
                            Value::Kosong => "NULL".to_string(),
                            _ => "''".to_string(),
                        };
                        vals.push(val_str);
                    }

                    format!(
                        "INSERT INTO {} ({}) VALUES ({})",
                        state.tabel,
                        cols.join(", "),
                        vals.join(", ")
                    )
                };

                // Reset state
                vm.get_heap_mut().db_query_state.tabel.clear();
                vm.get_heap_mut().db_query_state.kondisi.clear();

                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));

                let eksekusi_val = {
                    let mod_idx = vm.get_heap_mut().db_module_idx.unwrap();
                    let dict = vm.get_heap_mut().get_modul(mod_idx);
                    dict.get("eksekusi").cloned().unwrap()
                };

                vm.execute_function(eksekusi_val, vec![Value::String(sql_idx)])
            },
        ),
    };

    let perbarui_func = FungsiBawaanVM {
        nama: "perbarui".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.is_empty() {
                    return Err("db.perbarui membutuhkan 1 argumen: data kamus".to_string());
                }

                let sql = {
                    let state = vm.get_heap_mut().db_query_state.clone();
                    if state.tabel.is_empty() {
                        return Err("Panggil db.tabel() terlebih dahulu".to_string());
                    }

                    let kamus_idx = match &args[0] {
                        Value::Kamus(idx) => *idx,
                        _ => return Err("Data harus berupa Kamus".to_string()),
                    };

                    let kamus = vm.get_heap_mut().get_kamus(kamus_idx).clone();
                    if kamus.is_empty() {
                        return Err("Data kamus kosong".to_string());
                    }

                    let mut sets = Vec::new();

                    for (k, v) in kamus {
                        let val_str = match v {
                            Value::Angka(n) => n.to_string(),
                            Value::String(idx) => {
                                let s = vm.get_heap_mut().get_string(idx);
                                format!("'{}'", s.replace('\'', "''"))
                            }
                            Value::Boolean(b) => {
                                if b {
                                    "1".to_string()
                                } else {
                                    "0".to_string()
                                }
                            }
                            Value::Kosong => "NULL".to_string(),
                            _ => "''".to_string(),
                        };
                        sets.push(format!("{} = {}", k, val_str));
                    }

                                                        let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("UPDATE {} SET {}", table_name, sets.join(", "));

                    if !state.kondisi.is_empty() {
                        query.push_str(" WHERE ");
                        let mut conds = Vec::new();
                        for (k, o, v) in state.kondisi {
                            let val_str = match v {
                                Value::Angka(n) => n.to_string(),
                                Value::String(idx) => {
                                    let s = vm.get_heap_mut().get_string(idx);
                                    format!("'{}'", s.replace('\'', "''"))
                                }
                                Value::Boolean(b) => {
                                    if b {
                                        "1".to_string()
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                Value::Kosong => "NULL".to_string(),
                                _ => "''".to_string(),
                            };
                            conds.push(format!("{} {} {}", k, o, val_str));
                        }
                        query.push_str(&conds.join(" AND "));
                    }
                    query
                };

                vm.get_heap_mut().db_query_state.tabel.clear();
                vm.get_heap_mut().db_query_state.kondisi.clear();

                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));

                let eksekusi_val = {
                    let mod_idx = vm.get_heap_mut().db_module_idx.unwrap();
                    let dict = vm.get_heap_mut().get_modul(mod_idx);
                    dict.get("eksekusi").cloned().unwrap()
                };

                vm.execute_function(eksekusi_val, vec![Value::String(sql_idx)])
            },
        ),
    };

    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: std::sync::Arc::new(
            move |vm: &mut dyn VmContext, _args: Vec<Value>| -> Result<Value, String> {
                let sql = {
                    let state = vm.get_heap_mut().db_query_state.clone();
                    if state.tabel.is_empty() {
                        return Err("Panggil db.tabel() terlebih dahulu".to_string());
                    }

                                                        let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("DELETE FROM {}", table_name);

                    if !state.kondisi.is_empty() {
                        query.push_str(" WHERE ");
                        let mut conds = Vec::new();
                        for (k, o, v) in state.kondisi {
                            let val_str = match v {
                                Value::Angka(n) => n.to_string(),
                                Value::String(idx) => {
                                    let s = vm.get_heap_mut().get_string(idx);
                                    format!("'{}'", s.replace('\'', "''"))
                                }
                                Value::Boolean(b) => {
                                    if b {
                                        "1".to_string()
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                Value::Kosong => "NULL".to_string(),
                                _ => "''".to_string(),
                            };
                            conds.push(format!("{} {} {}", k, o, val_str));
                        }
                        query.push_str(&conds.join(" AND "));
                    }
                    query
                };

                vm.get_heap_mut().db_query_state.tabel.clear();
                vm.get_heap_mut().db_query_state.kondisi.clear();

                let sql_idx = vm.get_heap_mut().alloc(HeapData::String(sql));

                let eksekusi_val = {
                    let mod_idx = vm.get_heap_mut().db_module_idx.unwrap();
                    let dict = vm.get_heap_mut().get_modul(mod_idx);
                    dict.get("eksekusi").cloned().unwrap()
                };

                vm.execute_function(eksekusi_val, vec![Value::String(sql_idx)])
            },
        ),
    };

    let tabel_idx = vm.heap.alloc(HeapData::FungsiBawaan(tabel_func));
    let dimana_idx = vm.heap.alloc(HeapData::FungsiBawaan(dimana_func));
    let ambil_idx = vm.heap.alloc(HeapData::FungsiBawaan(ambil_func));
    let simpan_idx = vm.heap.alloc(HeapData::FungsiBawaan(simpan_func));
    let perbarui_idx = vm.heap.alloc(HeapData::FungsiBawaan(perbarui_func));
    let hapus_idx = vm.heap.alloc(HeapData::FungsiBawaan(hapus_func));

    module_dict.insert("tabel".to_string(), Value::FungsiBawaan(tabel_idx));
    module_dict.insert("dimana".to_string(), Value::FungsiBawaan(dimana_idx));
    module_dict.insert("ambil".to_string(), Value::FungsiBawaan(ambil_idx));
    module_dict.insert("simpan".to_string(), Value::FungsiBawaan(simpan_idx));
    module_dict.insert("perbarui".to_string(), Value::FungsiBawaan(perbarui_idx));
    module_dict.insert("hapus".to_string(), Value::FungsiBawaan(hapus_idx));

    let modul_idx = vm.heap.alloc(HeapData::Modul(module_dict));
    vm.heap.db_module_idx = Some(modul_idx);
    vm.environments
        .last_mut()
        .unwrap()
        .insert("db".to_string(), Value::Modul(modul_idx));
}


pub fn eksekusi_helper(vm: &mut dyn VmContext, pool: &DbPool, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("eksekusi membutuhkan minimal 1 argumen: SQL".to_string());
    }

    let sql = match &args[0] {
        Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
        _ => return Err("Argumen SQL harus berupa teks".to_string()),
    };

    let has_params = args.len() > 1 && matches!(&args[1], Value::Array(_));

    let sqlite_params: Vec<rusqlite::types::Value> = if has_params {
        if let Value::Array(arr_idx) = &args[1] {
            let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
            arr.iter()
                .map(|val| match val {
                    Value::Angka(n) => rusqlite::types::Value::Real(*n),
                    Value::String(idx) => rusqlite::types::Value::Text(
                        vm.get_heap_mut().get_string(*idx).clone(),
                    ),
                    Value::Boolean(b) => {
                        rusqlite::types::Value::Integer(if *b { 1 } else { 0 })
                    }
                    Value::Kosong => rusqlite::types::Value::Null,
                    _ => rusqlite::types::Value::Text(val.to_string(vm.get_heap_mut())),
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let string_params: Vec<String> = if has_params {
        if let Value::Array(arr_idx) = &args[1] {
            let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
            arr.iter()
                .map(|val| match val {
                    Value::Angka(n) => n.to_string(),
                    Value::String(idx) => format!(
                        "'{}'",
                        vm.get_heap_mut().get_string(*idx).replace('\'', "''")
                    ),
                    Value::Boolean(b) => {
                        if *b {
                            "1".to_string()
                        } else {
                            "0".to_string()
                        }
                    }
                    Value::Kosong => "NULL".to_string(),
                    _ => "''".to_string(),
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let start = std::time::Instant::now();
    let provider = pool.provider_name().to_string();

    let affected = pool.with_conn(|conn| {
        match conn {
            DatabaseConnection::Sqlite(c) => {
                let affected = c
                    .execute(&sql, rusqlite::params_from_iter(sqlite_params))
                    .map_err(|e| format!("SQLite Error: {}", e))?;
                Ok(affected as f64)
            }
            DatabaseConnection::Mysql(c) => {
                use mysql::prelude::Queryable;
                let mut final_sql = sql.clone();
                for val_str in &string_params {
                    final_sql = final_sql.replacen('?', val_str, 1);
                }
                c.query_drop(&final_sql)
                    .map_err(|e| format!("MySQL Error: {}", e))?;
                Ok(c.affected_rows() as f64)
            }
            DatabaseConnection::Postgres(c) => {
                let mut final_sql = sql.clone();
                for val_str in &string_params {
                    final_sql = final_sql.replacen('?', val_str, 1);
                }
                c.execute(&final_sql, &[])
                    .map_err(|e| format!("Postgres Error: {}", e))?;
                Ok(0.0)
            }
        }
    })?;

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let caller = format!(
        "{}:{}",
        vm.current_function_info().0,
        vm.current_lokasi().map(|l| l.baris).unwrap_or(0)
    );
    super::dev_dashboard::record_db_query(super::dev_dashboard::DbQueryTelemetry {
        sql,
        duration_ms,
        rows: 0,
        affected: affected as usize,
        provider,
        caller,
        timestamp: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S.%3f")
            .to_string(),
    });

    Ok(Value::Angka(affected))
}

#[derive(Clone)]
enum DbValue {
    Null,
    Int(i64),
    Float(f64),
    Text(String),
}

pub fn kueri_helper(vm: &mut dyn VmContext, pool: &DbPool, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("kueri membutuhkan minimal 1 argumen: SQL".to_string());
    }

    let sql = match &args[0] {
        Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
        _ => return Err("Argumen SQL harus berupa teks".to_string()),
    };

    let has_params = args.len() > 1 && matches!(&args[1], Value::Array(_));

    let sqlite_params: Vec<rusqlite::types::Value> = if has_params {
        if let Value::Array(arr_idx) = &args[1] {
            let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
            arr.iter()
                .map(|val| match val {
                    Value::Angka(n) => rusqlite::types::Value::Real(*n),
                    Value::String(idx) => rusqlite::types::Value::Text(
                        vm.get_heap_mut().get_string(*idx).clone(),
                    ),
                    Value::Boolean(b) => {
                        rusqlite::types::Value::Integer(if *b { 1 } else { 0 })
                    }
                    Value::Kosong => rusqlite::types::Value::Null,
                    _ => rusqlite::types::Value::Text(val.to_string(vm.get_heap_mut())),
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let string_params: Vec<String> = if has_params {
        if let Value::Array(arr_idx) = &args[1] {
            let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
            arr.iter()
                .map(|val| match val {
                    Value::Angka(n) => n.to_string(),
                    Value::String(idx) => format!(
                        "'{}'",
                        vm.get_heap_mut().get_string(*idx).replace('\'', "''")
                    ),
                    Value::Boolean(b) => {
                        if *b {
                            "1".to_string()
                        } else {
                            "0".to_string()
                        }
                    }
                    Value::Kosong => "NULL".to_string(),
                    _ => "''".to_string(),
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let start = std::time::Instant::now();
    let provider = pool.provider_name().to_string();

    let intermediate_results: Vec<std::collections::HashMap<String, DbValue>> =
        pool.with_conn(|conn| match conn {
            DatabaseConnection::Sqlite(c) => {
                let mut stmt = c
                    .prepare(&sql)
                    .map_err(|e| format!("SQLite Error: {}", e))?;
                let cols: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt
                    .query(rusqlite::params_from_iter(sqlite_params))
                    .map_err(|e| format!("SQLite Error: {}", e))?;
                let mut results = Vec::new();
                while let Some(row) =
                    rows.next().map_err(|e| format!("SQLite Error: {}", e))?
                {
                    let mut dict_vals = std::collections::HashMap::new();
                    for (i, col_name) in cols.iter().enumerate() {
                        let val: rusqlite::types::Value =
                            row.get(i).map_err(|e| format!("SQLite Error: {}", e))?;
                        let db_val = match val {
                            rusqlite::types::Value::Null => DbValue::Null,
                            rusqlite::types::Value::Integer(i) => DbValue::Int(i),
                            rusqlite::types::Value::Real(r) => DbValue::Float(r),
                            rusqlite::types::Value::Text(t) => DbValue::Text(t),
                            _ => DbValue::Null,
                        };
                        dict_vals.insert(col_name.clone(), db_val);
                    }
                    results.push(dict_vals);
                }
                Ok(results)
            }
            DatabaseConnection::Mysql(c) => {
                use mysql::prelude::Queryable;
                let mut final_sql = sql.clone();
                for val_str in &string_params {
                    final_sql = final_sql.replacen('?', val_str, 1);
                }
                let rows: Vec<mysql::Row> = c
                    .query(&final_sql)
                    .map_err(|e| format!("MySQL Error: {}", e))?;
                let mut results = Vec::new();
                for row in rows {
                    let mut dict_vals = std::collections::HashMap::new();
                    for col in row.columns().iter() {
                        let col_name = col.name_str().to_string();
                        let idx = row
                            .columns()
                            .iter()
                            .position(|c| c.name_str() == col_name)
                            .unwrap();
                        let db_val = match &row[idx] {
                            mysql::Value::NULL => DbValue::Null,
                            mysql::Value::Int(i) => DbValue::Int(*i),
                            mysql::Value::UInt(u) => DbValue::Int(*u as i64),
                            mysql::Value::Float(f) => DbValue::Float(*f as f64),
                            mysql::Value::Double(d) => DbValue::Float(*d),
                            mysql::Value::Bytes(b) => {
                                DbValue::Text(String::from_utf8_lossy(b).to_string())
                            }
                            _ => DbValue::Null,
                        };
                        dict_vals.insert(col_name, db_val);
                    }
                    results.push(dict_vals);
                }
                Ok(results)
            }
            DatabaseConnection::Postgres(c) => {
                let mut final_sql = sql.clone();
                for val_str in &string_params {
                    final_sql = final_sql.replacen('?', val_str, 1);
                }
                let rows = c
                    .query(&final_sql, &[])
                    .map_err(|e| format!("Postgres Error: {}", e))?;
                let mut results = Vec::new();
                for row in rows {
                    let mut dict_vals = std::collections::HashMap::new();
                    for (i, col) in row.columns().iter().enumerate() {
                        let col_name = col.name().to_string();
                        let db_val = if let Ok(s) = row.try_get::<_, String>(i) {
                            DbValue::Text(s)
                        } else if let Ok(n) = row.try_get::<_, i32>(i) {
                            DbValue::Int(n as i64)
                        } else if let Ok(n) = row.try_get::<_, i64>(i) {
                            DbValue::Int(n)
                        } else if let Ok(f) = row.try_get::<_, f64>(i) {
                            DbValue::Float(f)
                        } else {
                            DbValue::Null
                        };
                        dict_vals.insert(col_name, db_val);
                    }
                    results.push(dict_vals);
                }
                Ok(results)
            }
        })?;

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let caller = format!(
        "{}:{}",
        vm.current_function_info().0,
        vm.current_lokasi().map(|l| l.baris).unwrap_or(0)
    );
    super::dev_dashboard::record_db_query(super::dev_dashboard::DbQueryTelemetry {
        sql,
        duration_ms,
        rows: intermediate_results.len(),
        affected: 0,
        provider,
        caller,
        timestamp: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S.%3f")
            .to_string(),
    });

    let mut final_results = Vec::new();
    for row in intermediate_results {
        let mut rpl_dict = std::collections::HashMap::new();
        for (col_name, db_val) in row {
            let rpl_val = match db_val {
                DbValue::Null => Value::Kosong,
                DbValue::Int(i) => Value::Angka(i as f64),
                DbValue::Float(f) => Value::Angka(f),
                DbValue::Text(t) => {
                    let str_idx = vm.get_heap_mut().alloc(HeapData::String(t));
                    Value::String(str_idx)
                }
            };
            rpl_dict.insert(col_name, rpl_val);
        }
        let dict_idx = vm.get_heap_mut().alloc(HeapData::Kamus(rpl_dict));
        final_results.push(Value::Kamus(dict_idx));
    }

    let arr_idx = vm.get_heap_mut().alloc(HeapData::Array(final_results));
    Ok(Value::Array(arr_idx))
}

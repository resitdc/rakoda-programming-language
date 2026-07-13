use crate::heap::{DatabaseConnection, HeapData};
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use mysql::Conn as MysqlConnection;
use postgres::{Client as PostgresClient, NoTls};
use rusqlite::Connection as SqliteConnection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let hubungkan_func = FungsiBawaanVM {
        nama: "hubungkan".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
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
                _ => return Err("Koneksi harus berupa teks URL atau kamus konfigurasi".to_string()),
            };

            let conn = if url_str.starts_with("sqlite://") {
                let path = url_str.trim_start_matches("sqlite://");
                let c = SqliteConnection::open(path)
                    .map_err(|e| format!("Gagal koneksi SQLite: {}", e))?;
                DatabaseConnection::Sqlite(c)
            } else if url_str.starts_with("mysql://") {
                let opts = mysql::Opts::from_url(&url_str)
                    .map_err(|e| format!("URL MySQL tidak valid: {}", e))?;
                let c = MysqlConnection::new(opts)
                    .map_err(|e| format!("Gagal koneksi MySQL: {}", e))?;
                DatabaseConnection::Mysql(c)
            } else if url_str.starts_with("postgres://") {
                let c = PostgresClient::connect(&url_str, NoTls)
                    .map_err(|e| format!("Gagal koneksi Postgres: {}", e))?;
                DatabaseConnection::Postgres(c)
            } else {
                return Err(format!("Protokol tidak didukung: {}", url_str));
            };

            vm.get_heap_mut().db_connection = Some(Arc::new(Mutex::new(conn)));

            Ok(Value::Kosong)
        },
    };

    let eksekusi_func = FungsiBawaanVM {
        nama: "eksekusi".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("db.eksekusi membutuhkan minimal 1 argumen: SQL".to_string());
            }

            let sql = match &args[0] {
                Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen SQL harus berupa teks".to_string()),
            };

            let mut sqlite_params: Vec<rusqlite::types::Value> = vec![];
            if args.len() > 1
                && let Value::Array(arr_idx) = &args[1]
            {
                let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
                for val in arr {
                    sqlite_params.push(match val {
                        Value::Angka(n) => rusqlite::types::Value::Real(n),
                        Value::String(idx) => {
                            rusqlite::types::Value::Text(vm.get_heap_mut().get_string(idx).clone())
                        }
                        Value::Boolean(b) => rusqlite::types::Value::Integer(if b { 1 } else { 0 }),
                        Value::Kosong => rusqlite::types::Value::Null,
                        _ => rusqlite::types::Value::Text(val.to_string(vm.get_heap_mut())),
                    });
                }
            }

            let conn_mutex = match &vm.get_heap_mut().db_connection {
                Some(c) => c.clone(),
                None => {
                    return Err(
                        "Koneksi database belum dibuka. Panggil db.hubungkan() terlebih dahulu"
                            .to_string(),
                    );
                }
            };

            let mut conn_lock = conn_mutex.lock().unwrap();

            let start = std::time::Instant::now();
            let provider = match &mut *conn_lock {
                DatabaseConnection::Sqlite(_) => "sqlite",
                DatabaseConnection::Mysql(_) => "mysql",
                DatabaseConnection::Postgres(_) => "postgres",
            }
            .to_string();

            let affected = match &mut *conn_lock {
                DatabaseConnection::Sqlite(c) => {
                    c.execute(&sql, rusqlite::params_from_iter(sqlite_params))
                        .map_err(|e| format!("SQLite Error: {}", e))? as f64
                }
                DatabaseConnection::Mysql(c) => {
                    use mysql::prelude::Queryable;
                    // Fallback to string replacement for now if params exist
                    let mut final_sql = sql.clone();
                    if args.len() > 1
                        && let Value::Array(arr_idx) = &args[1]
                    {
                        let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
                        for val in arr {
                            let val_str = match val {
                                Value::Angka(n) => n.to_string(),
                                Value::String(idx) => format!(
                                    "'{}'",
                                    vm.get_heap_mut().get_string(idx).replace("'", "''")
                                ),
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
                            final_sql = final_sql.replacen("?", &val_str, 1);
                        }
                    }
                    c.query_drop(&final_sql)
                        .map_err(|e| format!("MySQL Error: {}", e))?;
                    c.affected_rows() as f64
                }
                DatabaseConnection::Postgres(c) => {
                    // Fallback to string replacement for now if params exist
                    let mut final_sql = sql.clone();
                    if args.len() > 1
                        && let Value::Array(arr_idx) = &args[1]
                    {
                        let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
                        for val in arr {
                            let val_str = match val {
                                Value::Angka(n) => n.to_string(),
                                Value::String(idx) => format!(
                                    "'{}'",
                                    vm.get_heap_mut().get_string(idx).replace("'", "''")
                                ),
                                Value::Boolean(b) => {
                                    if b {
                                        "TRUE".to_string()
                                    } else {
                                        "FALSE".to_string()
                                    }
                                }
                                Value::Kosong => "NULL".to_string(),
                                _ => "''".to_string(),
                            };
                            final_sql = final_sql.replacen("?", &val_str, 1);
                        }
                    }
                    c.execute(&final_sql, &[])
                        .map_err(|e| format!("Postgres Error: {}", e))? as f64
                }
            };

            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            let caller = format!(
                "{}:{}",
                vm.current_function_info().0,
                vm.current_lokasi().map(|l| l.baris).unwrap_or(0)
            );
            super::dev_dashboard::record_db_query(super::dev_dashboard::DbQueryTelemetry {
                sql: sql.clone(),
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
        },
    };

    let kueri_func = FungsiBawaanVM {
        nama: "kueri".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("db.kueri membutuhkan minimal 1 argumen: SQL".to_string());
            }

            let sql = match &args[0] {
                Value::String(idx) => vm.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen SQL harus berupa teks".to_string()),
            };
            let mut sqlite_params: Vec<rusqlite::types::Value> = vec![];
            if args.len() > 1
                && let Value::Array(arr_idx) = &args[1]
            {
                let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
                for val in arr {
                    sqlite_params.push(match val {
                        Value::Angka(n) => rusqlite::types::Value::Real(n),
                        Value::String(idx) => {
                            rusqlite::types::Value::Text(vm.get_heap_mut().get_string(idx).clone())
                        }
                        Value::Boolean(b) => rusqlite::types::Value::Integer(if b { 1 } else { 0 }),
                        Value::Kosong => rusqlite::types::Value::Null,
                        _ => rusqlite::types::Value::Text(val.to_string(vm.get_heap_mut())),
                    });
                }
            }

            enum DbValue {
                Null,
                Int(i64),
                Float(f64),
                Text(String),
            }

            let mut intermediate_results: Vec<HashMap<String, DbValue>> = Vec::new();

            let start = std::time::Instant::now();
            let provider;

            {
                let conn_mutex =
                    match &vm.get_heap_mut().db_connection {
                        Some(c) => c.clone(),
                        None => return Err(
                            "Koneksi database belum dibuka. Panggil db.hubungkan() terlebih dahulu"
                                .to_string(),
                        ),
                    };
                let mut conn_lock = conn_mutex.lock().unwrap();
                provider = match &*conn_lock {
                    DatabaseConnection::Sqlite(_) => "sqlite",
                    DatabaseConnection::Mysql(_) => "mysql",
                    DatabaseConnection::Postgres(_) => "postgres",
                }
                .to_string();

                match &mut *conn_lock {
                    DatabaseConnection::Sqlite(c) => {
                        let mut stmt = c
                            .prepare(&sql)
                            .map_err(|e| format!("SQLite Error: {}", e))?;
                        let cols: Vec<String> =
                            stmt.column_names().iter().map(|s| s.to_string()).collect();
                        let mut rows = stmt
                            .query(rusqlite::params_from_iter(sqlite_params))
                            .map_err(|e| format!("SQLite Error: {}", e))?;
                        while let Some(row) =
                            rows.next().map_err(|e| format!("SQLite Error: {}", e))?
                        {
                            let mut dict_vals = HashMap::new();
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
                            intermediate_results.push(dict_vals);
                        }
                    }
                    DatabaseConnection::Mysql(c) => {
                        use mysql::prelude::Queryable;
                        let mut final_sql = sql.clone();
                        if args.len() > 1
                            && let Value::Array(arr_idx) = &args[1]
                        {
                            let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
                            for val in arr {
                                let val_str = match val {
                                    Value::Angka(n) => n.to_string(),
                                    Value::String(idx) => format!(
                                        "'{}'",
                                        vm.get_heap_mut().get_string(idx).replace("'", "''")
                                    ),
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
                                final_sql = final_sql.replacen("?", &val_str, 1);
                            }
                        }
                        let rows: Vec<mysql::Row> = c
                            .query(&final_sql)
                            .map_err(|e| format!("MySQL Error: {}", e))?;
                        for row in rows {
                            let mut dict_vals = HashMap::new();
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
                            intermediate_results.push(dict_vals);
                        }
                    }
                    DatabaseConnection::Postgres(c) => {
                        let mut final_sql = sql.clone();
                        if args.len() > 1
                            && let Value::Array(arr_idx) = &args[1]
                        {
                            let arr = vm.get_heap_mut().get_array(*arr_idx).clone();
                            for val in arr {
                                let val_str = match val {
                                    Value::Angka(n) => n.to_string(),
                                    Value::String(idx) => format!(
                                        "'{}'",
                                        vm.get_heap_mut().get_string(idx).replace("'", "''")
                                    ),
                                    Value::Boolean(b) => {
                                        if b {
                                            "TRUE".to_string()
                                        } else {
                                            "FALSE".to_string()
                                        }
                                    }
                                    Value::Kosong => "NULL".to_string(),
                                    _ => "''".to_string(),
                                };
                                final_sql = final_sql.replacen("?", &val_str, 1);
                            }
                        }
                        let rows = c
                            .query(&final_sql, &[])
                            .map_err(|e| format!("Postgres Error: {}", e))?;
                        for row in rows {
                            let mut dict_vals = HashMap::new();
                            for (i, col) in row.columns().iter().enumerate() {
                                let col_name = col.name().to_string();
                                // Postgres type matching is tricky, we try text first, if not try i32 etc.
                                // Simplest is to get everything as string and let RPL handle it, but we can try basic types:
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
                            intermediate_results.push(dict_vals);
                        }
                    }
                };
            }

            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            let caller = format!(
                "{}:{}",
                vm.current_function_info().0,
                vm.current_lokasi().map(|l| l.baris).unwrap_or(0)
            );
            super::dev_dashboard::record_db_query(super::dev_dashboard::DbQueryTelemetry {
                sql: sql.clone(),
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
                let mut rpl_dict = HashMap::new();
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
        },
    };

    let hubungkan_idx = vm.heap.alloc(HeapData::FungsiBawaan(hubungkan_func));
    let eksekusi_idx = vm.heap.alloc(HeapData::FungsiBawaan(eksekusi_func));
    let kueri_idx = vm.heap.alloc(HeapData::FungsiBawaan(kueri_func));

    module_dict.insert("hubungkan".to_string(), Value::FungsiBawaan(hubungkan_idx));
    module_dict.insert("eksekusi".to_string(), Value::FungsiBawaan(eksekusi_idx));
    module_dict.insert("kueri".to_string(), Value::FungsiBawaan(kueri_idx));

    let tabel_func = FungsiBawaanVM {
        nama: "tabel".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
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
    };

    let dimana_func = FungsiBawaanVM {
        nama: "dimana".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
            if args.len() < 3 {
                return Err("db.dimana membutuhkan 3 argumen: kolom, operator, nilai".to_string());
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
    };

    let ambil_func = FungsiBawaanVM {
        nama: "ambil".to_string(),
        func: |vm: &mut dyn VmContext, _args: Vec<Value>| {
            let sql = {
                let state = vm.get_heap_mut().db_query_state.clone();
                if state.tabel.is_empty() {
                    return Err("Panggil db.tabel() terlebih dahulu".to_string());
                }

                let mut query = format!("SELECT * FROM {}", state.tabel);

                if !state.kondisi.is_empty() {
                    query.push_str(" WHERE ");
                    let mut conds = Vec::new();
                    for (k, o, v) in state.kondisi {
                        let val_str = match v {
                            Value::Angka(n) => n.to_string(),
                            Value::String(idx) => {
                                let s = vm.get_heap_mut().get_string(idx);
                                format!("'{}'", s.replace("'", "''"))
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
            // But we can just use the execute_function!
            let kueri_val = {
                let mod_idx = vm.get_heap_mut().db_module_idx.unwrap();
                let dict = vm.get_heap_mut().get_modul(mod_idx);
                dict.get("kueri").cloned().unwrap()
            };

            vm.execute_function(kueri_val, vec![Value::String(sql_idx)])
        },
    };

    let simpan_func = FungsiBawaanVM {
        nama: "simpan".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
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
                            format!("'{}'", s.replace("'", "''"))
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

                let query = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    state.tabel,
                    cols.join(", "),
                    vals.join(", ")
                );
                query
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
    };

    let perbarui_func = FungsiBawaanVM {
        nama: "perbarui".to_string(),
        func: |vm: &mut dyn VmContext, args: Vec<Value>| {
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
                            format!("'{}'", s.replace("'", "''"))
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

                let mut query = format!("UPDATE {} SET {}", state.tabel, sets.join(", "));

                if !state.kondisi.is_empty() {
                    query.push_str(" WHERE ");
                    let mut conds = Vec::new();
                    for (k, o, v) in state.kondisi {
                        let val_str = match v {
                            Value::Angka(n) => n.to_string(),
                            Value::String(idx) => {
                                let s = vm.get_heap_mut().get_string(idx);
                                format!("'{}'", s.replace("'", "''"))
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
    };

    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: |vm: &mut dyn VmContext, _args: Vec<Value>| {
            let sql = {
                let state = vm.get_heap_mut().db_query_state.clone();
                if state.tabel.is_empty() {
                    return Err("Panggil db.tabel() terlebih dahulu".to_string());
                }

                let mut query = format!("DELETE FROM {}", state.tabel);

                if !state.kondisi.is_empty() {
                    query.push_str(" WHERE ");
                    let mut conds = Vec::new();
                    for (k, o, v) in state.kondisi {
                        let val_str = match v {
                            Value::Angka(n) => n.to_string(),
                            Value::String(idx) => {
                                let s = vm.get_heap_mut().get_string(idx);
                                format!("'{}'", s.replace("'", "''"))
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

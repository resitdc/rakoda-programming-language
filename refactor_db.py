import re

with open('crates/vm/src/stdlib/db.rs', 'r') as f:
    content = f.read()

# We need to extract the logic of eksekusi_func and kueri_func.
# I will just write the helper functions as text and insert them at the end.
helpers = """

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
                        vm.get_heap_mut().get_string(*idx).replace('\\'', "''")
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
                        vm.get_heap_mut().get_string(*idx).replace('\\'', "''")
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
"""

with open('crates/vm/src/stdlib/db.rs', 'a') as f:
    f.write(helpers)


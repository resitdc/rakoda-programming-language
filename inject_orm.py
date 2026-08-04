import re

with open('crates/vm/src/stdlib/db.rs', 'r') as f:
    content = f.read()

# 1. Update hubungkan_func to include `query` and `table` methods
hubungkan_replacement = """                let kueri_func = FungsiBawaanVM {
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
                            kondisi: vec![],
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
                                
                                if let HeapData::QueryState(ref mut state) = &mut vm.get_heap_mut().objects[state_idx].data {
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
                                    let mut query = format!("SELECT * FROM {}", state.tabel);
                                    if !state.kondisi.is_empty() {
                                        query.push_str(" WHERE ");
                                        let mut conds = Vec::new();
                                        for (k, o, v) in state.kondisi {
                                            let val_str = match v {
                                                Value::Angka(n) => n.to_string(),
                                                Value::String(idx) => {
                                                    let s = vm.get_heap_mut().get_string(idx);
                                                    format!("'{}'", s.replace('\\'', "''"))
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
                                kueri_helper(vm, &pool, &[Value::String(sql_idx)])
                            }),
                        };
                        let select_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(select_func));

                        let insert_func = FungsiBawaanVM {
                            nama: "insert".to_string(),
                            func: std::sync::Arc::new(move |vm: &mut dyn VmContext, args: Vec<Value>| {
                                if args.is_empty() { return Err("insert butuh argumen kamus".to_string()); }
                                let dict_idx = match args[0] {
                                    Value::Kamus(idx) => idx,
                                    _ => return Err("insert butuh kamus".to_string()),
                                };
                                let sql = {
                                    let data = vm.get_heap_mut().get_kamus(dict_idx).clone();
                                    let state = match &vm.get_heap_mut().objects[state_idx].data {
                                        HeapData::QueryState(s) => s.clone(),
                                        _ => return Err("Query state tidak valid".to_string()),
                                    };
                                    let mut cols = Vec::new();
                                    let mut vals = Vec::new();
                                    for (k, v) in data {
                                        cols.push(k);
                                        let val_str = match v {
                                            Value::Angka(n) => n.to_string(),
                                            Value::String(idx) => {
                                                let s = vm.get_heap_mut().get_string(idx);
                                                format!("'{}'", s.replace('\\'', "''"))
                                            }
                                            Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
                                            Value::Kosong => "NULL".to_string(),
                                            _ => "''".to_string(),
                                        };
                                        vals.push(val_str);
                                    }
                                    format!("INSERT INTO {} ({}) VALUES ({})", state.tabel, cols.join(", "), vals.join(", "))
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
                                    let mut query = format!("DELETE FROM {}", state.tabel);
                                    if !state.kondisi.is_empty() {
                                        query.push_str(" WHERE ");
                                        let mut conds = Vec::new();
                                        for (k, o, v) in state.kondisi {
                                            let val_str = match v {
                                                Value::Angka(n) => n.to_string(),
                                                Value::String(idx) => {
                                                    let s = vm.get_heap_mut().get_string(idx);
                                                    format!("'{}'", s.replace('\\'', "''"))
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
                                                format!("'{}'", s.replace('\\'', "''"))
                                            }
                                            Value::Boolean(b) => if b { "1".to_string() } else { "0".to_string() },
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
                                                    format!("'{}'", s.replace('\\'', "''"))
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

                        if let HeapData::Kamus(ref mut d) = &mut vm.get_heap_mut().objects[qb_dict_idx].data {
                            d.insert("where".to_string(), Value::FungsiBawaan(where_idx));
                            d.insert("select".to_string(), Value::FungsiBawaan(select_idx));
                            d.insert("insert".to_string(), Value::FungsiBawaan(insert_idx));
                            d.insert("update".to_string(), Value::FungsiBawaan(update_idx));
                            d.insert("delete".to_string(), Value::FungsiBawaan(delete_idx));
                        }

                        Ok(Value::Kamus(qb_dict_idx))
                    }),
                };
                let table_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(table_func));
                dict.insert("table".to_string(), Value::FungsiBawaan(table_idx));
"""
content = re.sub(r'                let kueri_func = FungsiBawaanVM \{.*?dict\.insert\("kueri"\.to_string\(\), Value::FungsiBawaan\(kueri_idx\)\);', hubungkan_replacement, content, flags=re.MULTILINE | re.DOTALL)

# 2. Add `query` global alias and `db.Model()` implementation at the end of module_dict insertions
module_additions = """    module_dict.insert("query".to_string(), Value::FungsiBawaan(eksekusi_idx));

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
"""
content = re.sub(r'    module_dict\.insert\("kueri"\.to_string\(\), Value::FungsiBawaan\(kueri_idx\)\);', r'    module_dict.insert("kueri".to_string(), Value::FungsiBawaan(kueri_idx));\n' + module_additions, content)

with open('crates/vm/src/stdlib/db.rs', 'w') as f:
    f.write(content)

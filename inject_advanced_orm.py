import re

with open('crates/vm/src/stdlib/db.rs', 'r') as f:
    content = f.read()

# Replace DbQueryState initialization
qstate_replacement = """                        let query_state = DbQueryState {
                            tabel,
                            schema: None,
                            kondisi: vec![],
                            limit_val: None,
                            offset_val: None,
                        };"""
content = re.sub(r'                        let query_state = DbQueryState \{\s*tabel,\s*kondisi: vec!\[\],\s*\};', qstate_replacement, content)

# Find the end of QueryBuilder methods and inject new methods
# In `table_func`, we have:
#                         if let HeapData::Kamus(d) = &mut vm.get_heap_mut().objects[qb_dict_idx].data {
#                             d.insert("where".to_string(), Value::FungsiBawaan(where_idx));
#                             ...
#                             d.insert("delete".to_string(), Value::FungsiBawaan(delete_idx));
#                         }

# I need to insert the new function definitions before this block, and then update this block.

new_methods = """
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
                                                format!("'{}'", s.replace('\\'', "''"))
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
"""

# Replace the block assigning properties to qb_dict
dict_insertion_target = """                        if let HeapData::Kamus(d) = &mut vm.get_heap_mut().objects[qb_dict_idx].data {
                            d.insert("where".to_string(), Value::FungsiBawaan(where_idx));
                            d.insert("select".to_string(), Value::FungsiBawaan(select_idx));
                            d.insert("insert".to_string(), Value::FungsiBawaan(insert_idx));
                            d.insert("update".to_string(), Value::FungsiBawaan(update_idx));
                            d.insert("delete".to_string(), Value::FungsiBawaan(delete_idx));
                        }"""

dict_insertion_replacement = new_methods + """
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
"""
content = content.replace(dict_insertion_target, dict_insertion_replacement)

# Update `select`, `insert`, `update`, `delete` to handle table_name (with schema) and limit/offset.

def replace_table_logic(query_type):
    # Regex to find `let mut query = format!("... FROM {}", state.tabel);`
    # and replace with schema logic.
    pass

# For select
select_replacement = """                                    let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("SELECT * FROM {}", table_name);"""
content = re.sub(r'let mut query = format!\("SELECT \* FROM \{\}", state\.tabel\);', select_replacement, content)

select_limit_replacement = """                                        query.push_str(&conds.join(" AND "));
                                    }
                                    if let Some(l) = state.limit_val {
                                        query.push_str(&format!(" LIMIT {}", l));
                                    }
                                    if let Some(o) = state.offset_val {
                                        query.push_str(&format!(" OFFSET {}", o));
                                    }"""
content = re.sub(r'                                        query\.push_str\(&conds\.join\(" AND "\)\);\n                                    \}', select_limit_replacement, content, count=1)

# For update
update_replacement = """                                    let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("UPDATE {} SET {}", table_name, sets.join(", "));"""
content = re.sub(r'let mut query = format!\("UPDATE \{\} SET \{\}", state\.tabel, sets\.join\(", "\)\);', update_replacement, content)

# For delete
delete_replacement = """                                    let table_name = match state.schema {
                                        Some(s) => format!("{}.{}", s, state.tabel),
                                        None => state.tabel,
                                    };
                                    let mut query = format!("DELETE FROM {}", table_name);"""
content = re.sub(r'let mut query = format!\("DELETE FROM \{\}", state\.tabel\);', delete_replacement, content)

# For insert (handle array of dictionaries and single dictionary)
insert_replacement = """                        let insert_func = FungsiBawaanVM {
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
                                                    format!("'{}'", s.replace('\\'', "''"))
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
                        };"""
content = re.sub(r'                        let insert_func = FungsiBawaanVM \{.*?alloc\(HeapData::FungsiBawaan\(insert_func\)\);', insert_replacement, content, flags=re.MULTILINE | re.DOTALL)

with open('crates/vm/src/stdlib/db.rs', 'w') as f:
    f.write(content)

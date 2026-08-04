import re
import os

with open('crates/vm/src/stdlib/db.rs', 'r') as f:
    content = f.read()

# Add OnceLock for DB_RT at the top
if 'DB_RT' not in content:
    content = "static DB_RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();\n\npub fn get_db_rt() -> &'static tokio::runtime::Runtime {\n    DB_RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())\n}\n\n" + content

# Patch hubungkan
mssql_logic = """                } else if url_str.starts_with("mssql://") {
                    let parsed = url::Url::parse(&url_str).map_err(|e| format!("URL MSSQL salah: {}", e))?;
                    let host = parsed.host_str().unwrap_or("localhost");
                    let port = parsed.port().unwrap_or(1433);
                    let db_name = parsed.path().trim_start_matches('/');
                    let user = parsed.username();
                    let pass = parsed.password().unwrap_or("");
                    let mut config = tiberius::Config::new();
                    config.host(host);
                    config.port(port);
                    config.database(db_name);
                    if !user.is_empty() {
                        config.authentication(tiberius::AuthMethod::sql_server(user, pass));
                    }
                    config.trust_cert();
                    
                    let rt = crate::stdlib::db::get_db_rt();
                    let client = rt.block_on(async {
                        use tokio_util::compat::TokioAsyncWriteCompatExt;
                        let tcp = tokio::net::TcpStream::connect(config.get_addr()).await.unwrap();
                        let tcp = tcp.compat_write();
                        tiberius::Client::connect(config, tcp).await.unwrap()
                    });
                    DbPool::Mssql(std::sync::Arc::new(tokio::sync::Mutex::new(client)))
                } else {"""
content = content.replace("                } else {", mssql_logic, 1)

# Patch eksekusi_helper
mssql_exec = """            DatabaseConnection::Postgres(c) => {
                let mut final_sql = sql.clone();
                for val_str in &string_params {
                    final_sql = final_sql.replacen('?', val_str, 1);
                }
                c.execute(&final_sql, &[])
                    .map_err(|e| format!("Postgres Error: {}", e))?;
                Ok(0.0)
            }
            DatabaseConnection::Mssql(client) => {
                let rt = crate::stdlib::db::get_db_rt();
                let affected = rt.block_on(async {
                    let mut final_sql = sql.clone();
                    // Replace ? with @p1, @p2 etc, or just replace with string_params since we don't have prepared statement param passing easy with Tiberius raw sql
                    for val_str in &string_params {
                        final_sql = final_sql.replacen('?', val_str, 1);
                    }
                    let mut lock = client.lock().await;
                    let res = lock.execute(&final_sql, &[]).await.map_err(|e| format!("MSSQL Error: {}", e))?;
                    Ok::<f64, String>(res.total() as f64)
                })?;
                Ok(affected)
            }"""
content = content.replace("""            DatabaseConnection::Postgres(c) => {
                let mut final_sql = sql.clone();
                for val_str in &string_params {
                    final_sql = final_sql.replacen('?', val_str, 1);
                }
                c.execute(&final_sql, &[])
                    .map_err(|e| format!("Postgres Error: {}", e))?;
                Ok(0.0)
            }""", mssql_exec)

# Patch kueri_helper
mssql_query = """            DatabaseConnection::Postgres(c) => {
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
            DatabaseConnection::Mssql(client) => {
                let rt = crate::stdlib::db::get_db_rt();
                let results = rt.block_on(async {
                    let mut final_sql = sql.clone();
                    for val_str in &string_params {
                        final_sql = final_sql.replacen('?', val_str, 1);
                    }
                    let mut lock = client.lock().await;
                    let stream = lock.simple_query(&final_sql).await.map_err(|e| format!("MSSQL Error: {}", e))?;
                    let mut rows = stream.into_first_result().await.map_err(|e| format!("MSSQL Error: {}", e))?;
                    
                    let mut results = Vec::new();
                    for row in rows {
                        let mut dict_vals = std::collections::HashMap::new();
                        for (i, col) in row.columns().iter().enumerate() {
                            let col_name = col.name().to_string();
                            // Tiberius get() needs type inference, but we can check ColumnType
                            // For simplicity, we just use string representation for everything in MSSQL since tiberius typing is complex.
                            let db_val = if let Some(s) = row.get::<&str, _>(i) {
                                DbValue::Text(s.to_string())
                            } else if let Some(n) = row.get::<i32, _>(i) {
                                DbValue::Int(n as i64)
                            } else if let Some(n) = row.get::<i64, _>(i) {
                                DbValue::Int(n)
                            } else if let Some(f) = row.get::<f64, _>(i) {
                                DbValue::Float(f)
                            } else {
                                DbValue::Null
                            };
                            dict_vals.insert(col_name, db_val);
                        }
                        results.push(dict_vals);
                    }
                    Ok::<Vec<std::collections::HashMap<String, DbValue>>, String>(results)
                })?;
                Ok(results)
            }"""
content = content.replace("""            DatabaseConnection::Postgres(c) => {
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
            }""", mssql_query)

with open('crates/vm/src/stdlib/db.rs', 'w') as f:
    f.write(content)

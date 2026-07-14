use crate::value::{FungsiBawaanVM, FungsiVM, Value};
use mysql::Conn as MysqlConnection;
use postgres::Client as PostgresClient;
use r2d2::Pool as R2d2Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type SessionMap = Arc<Mutex<HashMap<String, (Option<std::time::Instant>, usize)>>>;

/// Koneksi database aktif (untuk operasi borrow)
pub enum DatabaseConnection<'a> {
    Sqlite(&'a mut rusqlite::Connection),
    Mysql(&'a mut MysqlConnection),
    Postgres(&'a mut PostgresClient),
}

/// Pool koneksi database. SQLite menggunakan r2d2 connection pool untuk
/// concurrent access, MySQL/Postgres masih single-connection via Arc<Mutex<...>>.
#[derive(Clone)]
pub enum DbPool {
    Sqlite(R2d2Pool<SqliteConnectionManager>),
    Mysql(Arc<Mutex<MysqlConnection>>),
    Postgres(Arc<Mutex<PostgresClient>>),
}

impl DbPool {
    /// Mendapatkan koneksi dari pool (SQLite) atau lock (MySQL/Postgres).
    /// Closure `f` menerima `DatabaseConnection` dan mengembalikan Result<T, String>.
    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(DatabaseConnection<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        match self {
            DbPool::Sqlite(pool) => {
                let mut conn = pool
                    .get()
                    .map_err(|e| format!("Gagal mengambil koneksi dari pool: {}", e))?;
                f(DatabaseConnection::Sqlite(&mut conn))
            }
            DbPool::Mysql(conn_mutex) => {
                let mut guard = conn_mutex
                    .lock()
                    .map_err(|e| format!("Gagal lock MySQL: {}", e))?;
                f(DatabaseConnection::Mysql(&mut guard))
            }
            DbPool::Postgres(conn_mutex) => {
                let mut guard = conn_mutex
                    .lock()
                    .map_err(|e| format!("Gagal lock Postgres: {}", e))?;
                f(DatabaseConnection::Postgres(&mut guard))
            }
        }
    }

    /// Nama provider database
    pub fn provider_name(&self) -> &'static str {
        match self {
            DbPool::Sqlite(_) => "sqlite",
            DbPool::Mysql(_) => "mysql",
            DbPool::Postgres(_) => "postgres",
        }
    }

    /// Buat pool SQLite (r2d2) dengan path
    pub fn new_sqlite_pool(path: &str, pool_size: u32) -> Result<Self, String> {
        let manager = SqliteConnectionManager::file(path);
        let pool = R2d2Pool::builder()
            .max_size(pool_size)
            .build(manager)
            .map_err(|e| format!("Gagal membuat pool SQLite: {}", e))?;
        Ok(DbPool::Sqlite(pool))
    }
}

#[derive(Clone)]
pub struct HeapObject {
    pub is_marked: bool,
    pub data: HeapData,
}

#[derive(Clone)]
pub enum HeapData {
    String(String),
    Array(Vec<Value>),
    Kamus(HashMap<String, Value>),
    Fungsi(FungsiVM),
    FungsiBawaan(FungsiBawaanVM),
    Modul(HashMap<String, Value>),
    Free(usize), // Next free index
}

#[derive(Clone, Default)]
pub struct WebConfig {
    pub kompresi: bool,
    pub rate_limit: Option<u32>,
    pub proxies: HashMap<String, String>,
}

#[derive(Clone)]
pub struct WebState {
    pub sessions: SessionMap,
    pub active_session_id: Option<String>,
    pub active_cookies: HashMap<String, String>,
    pub cookies_to_set: Vec<String>,
}

impl Default for WebState {
    fn default() -> Self {
        Self {
            sessions: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            active_session_id: None,
            active_cookies: HashMap::new(),
            cookies_to_set: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct DbQueryState {
    pub tabel: String,
    pub kondisi: Vec<(String, String, Value)>, // kolom, operator, nilai
}

#[derive(Clone, Default)]
pub struct WebCache {
    pub templates_code: HashMap<String, String>,
    pub static_files: HashMap<String, (String, Vec<u8>)>,
}

#[derive(Clone)]
pub struct Heap {
    pub objects: Vec<HeapObject>,
    pub free_list_head: Option<usize>,
    pub allocated_count: usize,
    pub web_routes: HashMap<String, HashMap<String, Value>>,
    pub web_static_dirs: HashMap<String, String>,
    pub web_config: WebConfig,
    pub web_state: WebState,
    pub web_cache: std::sync::Arc<std::sync::Mutex<WebCache>>,
    pub db_pool: Option<DbPool>,
    pub db_query_state: DbQueryState,
    pub db_module_idx: Option<usize>,
    /// Direktori root proyek (tempat file .rpl utama berada).
    /// Digunakan untuk resolve path relatif seperti template, database, file.
    pub project_root: Option<PathBuf>,
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            free_list_head: None,
            allocated_count: 0,
            web_routes: HashMap::new(),
            web_static_dirs: HashMap::new(),
            web_config: WebConfig::default(),
            web_state: WebState::default(),
            web_cache: std::sync::Arc::new(std::sync::Mutex::new(WebCache::default())),
            db_pool: None,
            db_query_state: DbQueryState::default(),
            db_module_idx: None,
            project_root: None,
        }
    }

    pub fn alloc(&mut self, data: HeapData) -> usize {
        self.allocated_count += 1;

        if let Some(idx) = self.free_list_head
            && let HeapData::Free(next) = self.objects[idx].data
        {
            self.free_list_head = if next == usize::MAX { None } else { Some(next) };
            self.objects[idx] = HeapObject {
                is_marked: false,
                data,
            };
            return idx;
        }

        let idx = self.objects.len();
        self.objects.push(HeapObject {
            is_marked: false,
            data,
        });
        idx
    }

    pub fn free(&mut self, idx: usize) {
        let next = self.free_list_head.unwrap_or(usize::MAX);
        self.objects[idx].data = HeapData::Free(next);
        self.free_list_head = Some(idx);
        self.allocated_count -= 1;
    }

    pub fn get_string(&self, idx: usize) -> &String {
        if let HeapData::String(s) = &self.objects[idx].data {
            s
        } else {
            panic!("Expected String at heap index {}", idx);
        }
    }

    pub fn get_array(&self, idx: usize) -> &Vec<Value> {
        if let HeapData::Array(arr) = &self.objects[idx].data {
            arr
        } else {
            panic!("Expected Array at heap index {}", idx);
        }
    }

    pub fn get_array_mut(&mut self, idx: usize) -> &mut Vec<Value> {
        if let HeapData::Array(arr) = &mut self.objects[idx].data {
            arr
        } else {
            panic!("Expected Array at heap index {}", idx);
        }
    }

    pub fn get_kamus(&self, idx: usize) -> &HashMap<String, Value> {
        if let HeapData::Kamus(k) = &self.objects[idx].data {
            k
        } else {
            panic!("Expected Kamus at heap index {}", idx);
        }
    }

    pub fn get_kamus_mut(&mut self, idx: usize) -> &mut HashMap<String, Value> {
        if let HeapData::Kamus(k) = &mut self.objects[idx].data {
            k
        } else {
            panic!("Expected Kamus at heap index {}", idx);
        }
    }

    pub fn get_modul(&self, idx: usize) -> &HashMap<String, Value> {
        if let HeapData::Modul(m) = &self.objects[idx].data {
            m
        } else {
            panic!("Expected Modul at heap index {}", idx);
        }
    }

    pub fn get_fungsi(&self, idx: usize) -> &FungsiVM {
        if let HeapData::Fungsi(f) = &self.objects[idx].data {
            f
        } else {
            panic!("Expected Fungsi at heap index {}", idx);
        }
    }

    pub fn get_fungsi_bawaan(&self, idx: usize) -> &FungsiBawaanVM {
        if let HeapData::FungsiBawaan(f) = &self.objects[idx].data {
            f
        } else {
            panic!("Expected FungsiBawaan at heap index {}", idx);
        }
    }

    pub fn mark(&mut self, idx: usize) {
        if self.objects[idx].is_marked {
            return;
        }
        self.objects[idx].is_marked = true;

        let mut worklist = vec![idx];

        while let Some(current) = worklist.pop() {
            let children = match &self.objects[current].data {
                HeapData::Array(arr) => {
                    let mut c = Vec::new();
                    for val in arr {
                        if let Value::Array(i) = val {
                            c.push(*i);
                        }
                        if let Value::Kamus(i) = val {
                            c.push(*i);
                        }
                        if let Value::String(i) = val {
                            c.push(*i);
                        }
                        if let Value::Fungsi(i, _) = val {
                            c.push(*i);
                        }
                        if let Value::FungsiBawaan(i) = val {
                            c.push(*i);
                        }
                        if let Value::Modul(i) = val {
                            c.push(*i);
                        }
                    }
                    c
                }
                HeapData::Kamus(k) | HeapData::Modul(k) => {
                    let mut c = Vec::new();
                    for val in k.values() {
                        if let Value::Array(i) = val {
                            c.push(*i);
                        }
                        if let Value::Kamus(i) = val {
                            c.push(*i);
                        }
                        if let Value::String(i) = val {
                            c.push(*i);
                        }
                        if let Value::Fungsi(i, _) = val {
                            c.push(*i);
                        }
                        if let Value::FungsiBawaan(i) = val {
                            c.push(*i);
                        }
                        if let Value::Modul(i) = val {
                            c.push(*i);
                        }
                    }
                    c
                }
                HeapData::Fungsi(f) => {
                    let mut c = Vec::new();
                    for val in &f.chunk.constants {
                        if let Value::Array(i) = val {
                            c.push(*i);
                        }
                        if let Value::Kamus(i) = val {
                            c.push(*i);
                        }
                        if let Value::String(i) = val {
                            c.push(*i);
                        }
                        if let Value::Fungsi(i, _) = val {
                            c.push(*i);
                        }
                        if let Value::FungsiBawaan(i) = val {
                            c.push(*i);
                        }
                        if let Value::Modul(i) = val {
                            c.push(*i);
                        }
                    }
                    c
                }
                _ => Vec::new(),
            };

            for child in children {
                if !self.objects[child].is_marked {
                    self.objects[child].is_marked = true;
                    worklist.push(child);
                }
            }
        }
    }

    pub fn mark_sessions_and_cache(&mut self) {
        let mut session_indices = Vec::new();
        if let Ok(sessions) = self.web_state.sessions.lock() {
            for (_, idx) in sessions.values() {
                session_indices.push(*idx);
            }
        }
        for idx in session_indices {
            self.mark(idx);
        }
    }

    pub fn sweep(&mut self) {
        for i in 0..self.objects.len() {
            if let HeapData::Free(_) = self.objects[i].data {
                continue;
            }
            if !self.objects[i].is_marked {
                self.free(i);
            } else {
                self.objects[i].is_marked = false;
            }
        }
    }
}
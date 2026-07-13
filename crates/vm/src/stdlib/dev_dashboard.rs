use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use tokio::sync::mpsc;

// Global sender registry for WebSocket connections
pub static WS_BROADCAST: OnceLock<Mutex<Vec<mpsc::UnboundedSender<String>>>> = OnceLock::new();

pub fn get_ws_broadcast() -> &'static Mutex<Vec<mpsc::UnboundedSender<String>>> {
    WS_BROADCAST.get_or_init(|| Mutex::new(Vec::new()))
}

// Broadcast helper
pub fn broadcast_message(msg: String) {
    if let Ok(mut senders) = get_ws_broadcast().lock() {
        senders.retain(|tx| tx.send(msg.clone()).is_ok());
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct LifecycleEvent {
    pub name: String,
    pub duration_ms: f64,
    pub memory_used_kb: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct RequestTelemetry {
    pub id: String,
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration_ms: f64,
    pub memory_used_kb: usize,
    pub response_size: usize,
    pub ip: String,
    pub user_agent: String,
    pub request_headers: HashMap<String, String>,
    pub response_headers: HashMap<String, String>,
    pub raw_body: String,
    pub query_params: HashMap<String, String>,
    pub route_params: HashMap<String, String>,
    pub lifecycle_events: Vec<LifecycleEvent>,
}

#[derive(Serialize, Clone, Debug)]
pub struct DbQueryTelemetry {
    pub sql: String,
    pub duration_ms: f64,
    pub rows: usize,
    pub affected: usize,
    pub provider: String,
    pub caller: String,
    pub timestamp: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct LogTelemetry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub caller: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DevTelemetry {
    pub requests: Vec<RequestTelemetry>,
    pub db_queries: Vec<DbQueryTelemetry>,
    pub logs: Vec<LogTelemetry>,
    pub memory_peak_kb: usize,
}

impl Default for DevTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl DevTelemetry {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            db_queries: Vec::new(),
            logs: Vec::new(),
            memory_peak_kb: 0,
        }
    }
}

pub static TELEMETRY: OnceLock<Mutex<DevTelemetry>> = OnceLock::new();

pub fn get_telemetry() -> &'static Mutex<DevTelemetry> {
    TELEMETRY.get_or_init(|| Mutex::new(DevTelemetry::new()))
}

// Telemetry injection APIs
pub fn record_request(req: RequestTelemetry) {
    if !is_dev_mode() {
        return;
    }

    let mut tel = get_telemetry().lock().unwrap();
    if req.memory_used_kb > tel.memory_peak_kb {
        tel.memory_peak_kb = req.memory_used_kb;
    }

    let req_clone = req.clone();
    tel.requests.push(req);

    // Broadcast event
    let payload = serde_json::json!({
        "event": "new_request",
        "data": req_clone
    })
    .to_string();
    broadcast_message(payload);
}

pub fn record_db_query(query: DbQueryTelemetry) {
    if !is_dev_mode() {
        return;
    }

    let mut tel = get_telemetry().lock().unwrap();
    let query_clone = query.clone();
    tel.db_queries.push(query);

    let payload = serde_json::json!({
        "event": "new_query",
        "data": query_clone
    })
    .to_string();
    broadcast_message(payload);
}

pub fn record_log(log_item: LogTelemetry) {
    if !is_dev_mode() {
        return;
    }

    let mut tel = get_telemetry().lock().unwrap();
    let log_clone = log_item.clone();
    tel.logs.push(log_item);

    let payload = serde_json::json!({
        "event": "new_log",
        "data": log_clone
    })
    .to_string();
    broadcast_message(payload);
}

pub fn is_dev_mode() -> bool {
    std::env::var("RPL_ENV").unwrap_or_else(|_| "development".to_string()) != "production"
}

// Dev Dashboard HTML dashboard page
pub const DASHBOARD_HTML: &str = include_str!("dev_dashboard.html");

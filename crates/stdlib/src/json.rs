use crate::DaftarFungsiRpl;
use crate::jenis::NilaiRpl;
use serde_json::Value;

/// Konversi NilaiRpl → serde_json::Value (bantuan internal)
fn ke_json(nilai: &NilaiRpl) -> Value {
    match nilai {
        NilaiRpl::Angka(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Value::Number(num)
            } else {
                Value::Null
            }
        }
        NilaiRpl::Teks(s) => Value::String(s.clone()),
        NilaiRpl::Boolean(b) => Value::Bool(*b),
        NilaiRpl::Kosong => Value::Null,
        NilaiRpl::Daftar(arr) => {
            let vec: Vec<Value> = arr.iter().map(ke_json).collect();
            Value::Array(vec)
        }
        NilaiRpl::Kamus(map) => {
            let mut obj_map = serde_json::Map::new();
            for (k, v) in map {
                obj_map.insert(k.clone(), ke_json(v));
            }
            Value::Object(obj_map)
        }
    }
}

/// Konversi serde_json::Value → NilaiRpl (bantuan internal)
fn dari_json(val: &Value) -> NilaiRpl {
    match val {
        Value::Null => NilaiRpl::Kosong,
        Value::Bool(b) => NilaiRpl::Boolean(*b),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                NilaiRpl::Angka(f)
            } else {
                NilaiRpl::Kosong
            }
        }
        Value::String(s) => NilaiRpl::Teks(s.clone()),
        Value::Array(arr) => {
            let vec: Vec<NilaiRpl> = arr.iter().map(dari_json).collect();
            NilaiRpl::Daftar(vec)
        }
        Value::Object(map) => {
            let mut kamus = std::collections::HashMap::new();
            for (k, v) in map {
                kamus.insert(k.clone(), dari_json(v));
            }
            NilaiRpl::Kamus(kamus)
        }
    }
}

/// Fungsi-fungsi JSON murni (tidak tergantung engine).
pub fn fungsi_json() -> DaftarFungsiRpl {
    vec![("buat", buat_impl), ("parse", parse_impl)]
}

fn buat_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("json.buat membutuhkan 1 argumen: nilai".to_string());
    }
    let json_val = ke_json(&args[0]);
    match serde_json::to_string(&json_val) {
        Ok(s) => Ok(NilaiRpl::Teks(s)),
        Err(e) => Err(format!("json.buat gagal: {}", e)),
    }
}

fn parse_impl(args: &[NilaiRpl]) -> Result<NilaiRpl, String> {
    if args.is_empty() {
        return Err("json.parse membutuhkan 1 argumen: teks_json".to_string());
    }
    match &args[0] {
        NilaiRpl::Teks(s) => match serde_json::from_str::<Value>(s) {
            Ok(val) => Ok(dari_json(&val)),
            Err(e) => Err(format!("json.parse gagal: {}", e)),
        },
        _ => Err("json.parse hanya menerima teks".to_string()),
    }
}

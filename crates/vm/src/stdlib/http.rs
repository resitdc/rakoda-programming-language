use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;
use std::time::Instant;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.is_empty() {
                return Err("Fungsi 'get' membutuhkan minimal 1 argumen: url".to_string());
            }
            if let Value::String(idx) = &args[0] {
                let url = ctx.get_heap_mut().get_string(*idx).clone();
                let start = Instant::now();
                match ureq::get(&url).call() {
                    Ok(response) => {
                        let rd = stdlib::http::extract_response_data(response, start.elapsed());

                        let mut resp_dict = HashMap::new();
                        resp_dict.insert("status".to_string(), Value::Angka(rd.status as f64));
                        resp_dict.insert("status_text".to_string(), {
                            let idx = ctx.get_heap_mut().alloc(HeapData::String(rd.status_text));
                            Value::String(idx)
                        });
                        resp_dict.insert("berhasil".to_string(), Value::Boolean(rd.berhasil));
                        resp_dict.insert("waktu".to_string(), Value::Angka(rd.waktu_ms as f64));

                        let mut header_dict = HashMap::new();
                        for (k, v) in &rd.headers {
                            let v_idx = ctx.get_heap_mut().alloc(HeapData::String(v.clone()));
                            header_dict.insert(k.clone(), Value::String(v_idx));
                        }
                        let header_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(header_dict));
                        resp_dict.insert("header".to_string(), Value::Kamus(header_idx));

                        resp_dict.insert("ukuran".to_string(), Value::Angka(rd.ukuran as f64));
                        let body_idx = ctx.get_heap_mut().alloc(HeapData::String(rd.body));
                        resp_dict.insert("body".to_string(), Value::String(body_idx));

                        if let Some(data) = rd.data {
                            let data_idx = ctx.get_heap_mut().alloc(HeapData::String(data));
                            resp_dict.insert("data".to_string(), Value::String(data_idx));
                        } else {
                            resp_dict.insert("data".to_string(), Value::Kosong);
                        }

                        let dict_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(resp_dict));
                        Ok(Value::Kamus(dict_idx))
                    }
                    Err(e) => {
                        let rd = stdlib::http::build_error_response_data(
                            &format!("Permintaan HTTP gagal: {}", e),
                            start.elapsed(),
                        );
                        let mut err_dict = HashMap::new();
                        err_dict.insert("status".to_string(), Value::Angka(rd.status as f64));
                        err_dict.insert("status_text".to_string(), {
                            let idx = ctx.get_heap_mut().alloc(HeapData::String(rd.status_text));
                            Value::String(idx)
                        });
                        err_dict.insert("berhasil".to_string(), Value::Boolean(rd.berhasil));
                        err_dict.insert("waktu".to_string(), Value::Angka(rd.waktu_ms as f64));
                        let body_idx = ctx.get_heap_mut().alloc(HeapData::String(rd.body));
                        err_dict.insert("body".to_string(), Value::String(body_idx));
                        err_dict.insert("data".to_string(), Value::Kosong);

                        let dict_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(err_dict));
                        Ok(Value::Kamus(dict_idx))
                    }
                }
            } else {
                Err("URL harus berupa teks".to_string())
            }
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    module_dict.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("http".to_string(), Value::Kamus(dict_idx));
}
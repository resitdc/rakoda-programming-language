use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    // Helper to get or create session ID
    fn get_or_create_session(ctx: &mut dyn VmContext) -> String {
        if let Some(id) = &ctx.get_heap_mut().web_state.active_session_id {
            return id.clone();
        }
        let new_id = Uuid::new_v4().to_string();
        ctx.get_heap_mut().web_state.active_session_id = Some(new_id.clone());

        let cookie_str = format!("RPL_SESSIONID={}; Path=/; HttpOnly", new_id);
        ctx.get_heap_mut().web_state.cookies_to_set.push(cookie_str);

        new_id
    }

    let set_func = FungsiBawaanVM {
        nama: "set".to_string(),
        func: |ctx, args| {
            if args.len() != 2 {
                return Err("Fungsi 'session.set' membutuhkan 2 argumen: kunci, nilai".to_string());
            }
            if let Value::String(k_idx) = &args[0] {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();
                let val = args[1];

                let session_id = get_or_create_session(ctx);

                // Get or create session kamus
                let kamus_idx = if let Some((_, idx)) = ctx
                    .get_heap_mut()
                    .web_state
                    .sessions
                    .lock()
                    .unwrap()
                    .get(&session_id)
                {
                    *idx
                } else {
                    let new_kamus_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(HashMap::new()));
                    ctx.get_heap_mut()
                        .web_state
                        .sessions
                        .lock()
                        .unwrap()
                        .insert(session_id.clone(), (None, new_kamus_idx));
                    new_kamus_idx
                };

                ctx.get_heap_mut().get_kamus_mut(kamus_idx).insert(key, val);
                Ok(Value::Kosong)
            } else {
                Err("Kunci session harus berupa teks".to_string())
            }
        },
    };
    let set_idx = vm.heap.alloc(HeapData::FungsiBawaan(set_func));
    module_dict.insert("set".to_string(), Value::FungsiBawaan(set_idx));

    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'session.get' membutuhkan 1 argumen: kunci".to_string());
            }
            if let Value::String(k_idx) = &args[0] {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();

                let session_id_opt = ctx.get_heap_mut().web_state.active_session_id.clone();
                if let Some(sid) = session_id_opt {
                    let mut expired = false;
                    let mut k_idx = 0;
                    let mut found = false;

                    if let Some((exp_opt, kamus_idx)) = ctx
                        .get_heap_mut()
                        .web_state
                        .sessions
                        .lock()
                        .unwrap()
                        .get(&sid)
                    {
                        // Check if expired
                        if let Some(exp) = exp_opt
                            && Instant::now() > *exp
                        {
                            expired = true;
                        }
                        if !expired {
                            found = true;
                            k_idx = *kamus_idx;
                        }
                    }

                    if expired {
                        ctx.get_heap_mut()
                            .web_state
                            .sessions
                            .lock()
                            .unwrap()
                            .remove(&sid);
                        return Ok(Value::Kosong);
                    }

                    if found && let Some(val) = ctx.get_heap_mut().get_kamus(k_idx).get(&key) {
                        return Ok(*val);
                    }
                }
                Ok(Value::Kosong)
            } else {
                Err("Kunci session harus berupa teks".to_string())
            }
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    module_dict.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'session.hapus' membutuhkan 1 argumen: kunci".to_string());
            }
            if let Value::String(k_idx) = &args[0] {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();
                let session_id_opt = ctx.get_heap_mut().web_state.active_session_id.clone();
                if let Some(sid) = session_id_opt {
                    let kamus_idx_opt = ctx
                        .get_heap_mut()
                        .web_state
                        .sessions
                        .lock()
                        .unwrap()
                        .get(&sid)
                        .map(|(_, idx)| *idx);
                    if let Some(idx) = kamus_idx_opt {
                        ctx.get_heap_mut().get_kamus_mut(idx).remove(&key);
                    }
                }
                Ok(Value::Kosong)
            } else {
                Err("Kunci session harus berupa teks".to_string())
            }
        },
    };
    let hapus_idx = vm.heap.alloc(HeapData::FungsiBawaan(hapus_func));
    module_dict.insert("hapus".to_string(), Value::FungsiBawaan(hapus_idx));

    let set_expired_func = FungsiBawaanVM {
        nama: "set_expired".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err(
                    "Fungsi 'session.set_expired' membutuhkan 1 argumen: durasi_detik".to_string(),
                );
            }
            let durasi = match &args[0] {
                Value::Angka(n) => *n as u64,
                _ => return Err("Durasi harus berupa angka".to_string()),
            };

            let session_id = get_or_create_session(ctx);
            let target_time = Instant::now() + Duration::from_secs(durasi);

            // update session store
            let sessions_arc = ctx.get_heap_mut().web_state.sessions.clone();
            let mut sessions = sessions_arc.lock().unwrap();
            if let Some((exp_opt, _)) = sessions.get_mut(&session_id) {
                *exp_opt = Some(target_time);
            } else {
                let new_kamus_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(HashMap::new()));
                sessions.insert(session_id.clone(), (Some(target_time), new_kamus_idx));
            }

            // update cookie expiration
            let cookie_str = format!(
                "RPL_SESSIONID={}; Path=/; HttpOnly; Max-Age={}",
                session_id, durasi
            );
            ctx.get_heap_mut().web_state.cookies_to_set.push(cookie_str);

            Ok(Value::Kosong)
        },
    };
    let set_expired_idx = vm.heap.alloc(HeapData::FungsiBawaan(set_expired_func));
    module_dict.insert(
        "set_expired".to_string(),
        Value::FungsiBawaan(set_expired_idx),
    );

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("session".to_string(), Value::Kamus(dict_idx));
}

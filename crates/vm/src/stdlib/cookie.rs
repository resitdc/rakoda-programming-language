use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let set_func = FungsiBawaanVM {
        nama: "set".to_string(),
        func: |ctx, args| {
            if args.len() < 2 {
                return Err(
                    "Fungsi 'cookie.set' membutuhkan minimal 2 argumen: nama, nilai".to_string(),
                );
            }
            if let (Value::String(k_idx), Value::String(v_idx)) = (&args[0], &args[1]) {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();
                let val = ctx.get_heap_mut().get_string(*v_idx).clone();

                let mut cookie_str = format!("{}={}; Path=/", key, val);

                if args.len() >= 3
                    && let Value::Angka(max_age) = &args[2] {
                        cookie_str.push_str(&format!("; Max-Age={}", *max_age as i64));
                    }

                ctx.get_heap_mut().web_state.cookies_to_set.push(cookie_str);
                Ok(Value::Kosong)
            } else {
                Err("Nama dan nilai cookie harus berupa teks".to_string())
            }
        },
    };
    let set_idx = vm.heap.alloc(HeapData::FungsiBawaan(set_func));
    module_dict.insert("set".to_string(), Value::FungsiBawaan(set_idx));

    let get_func = FungsiBawaanVM {
        nama: "get".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'cookie.get' membutuhkan 1 argumen: nama".to_string());
            }
            if let Value::String(k_idx) = &args[0] {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();

                let val_opt = ctx
                    .get_heap_mut()
                    .web_state
                    .active_cookies
                    .get(&key)
                    .cloned();
                if let Some(val) = val_opt {
                    let val_str = ctx.get_heap_mut().alloc(HeapData::String(val));
                    Ok(Value::String(val_str))
                } else {
                    Ok(Value::Kosong)
                }
            } else {
                Err("Nama cookie harus berupa teks".to_string())
            }
        },
    };
    let get_idx = vm.heap.alloc(HeapData::FungsiBawaan(get_func));
    module_dict.insert("get".to_string(), Value::FungsiBawaan(get_idx));

    let hapus_func = FungsiBawaanVM {
        nama: "hapus".to_string(),
        func: |ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'cookie.hapus' membutuhkan 1 argumen: nama".to_string());
            }
            if let Value::String(k_idx) = &args[0] {
                let key = ctx.get_heap_mut().get_string(*k_idx).clone();
                let cookie_str = format!("{}=; Path=/; Max-Age=0", key);
                ctx.get_heap_mut().web_state.cookies_to_set.push(cookie_str);
                Ok(Value::Kosong)
            } else {
                Err("Nama cookie harus berupa teks".to_string())
            }
        },
    };
    let hapus_idx = vm.heap.alloc(HeapData::FungsiBawaan(hapus_func));
    module_dict.insert("hapus".to_string(), Value::FungsiBawaan(hapus_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("cookie".to_string(), Value::Kamus(dict_idx));
}

#[cfg(feature = "enterprise")]
use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "enterprise")]
use libloading::{Library, Symbol};
#[cfg(feature = "enterprise")]
use std::ffi::CString;

#[allow(unused_mut)]
#[allow(unused_variables)]
pub fn register(vm: &mut VM) {
    let mut modul = HashMap::new();

    #[cfg(feature = "enterprise")]
    {
        let muat_lib_obj = FungsiBawaanVM {
            nama: "muat_lib".to_string(),
            func: Arc::new(|vm_ctx: &mut dyn VmContext, args: Vec<Value>| {
                if args.len() != 1 {
                    return Err("muat_lib memerlukan 1 argumen (path)".to_string());
                }
                let path = match &args[0] {
                    Value::String(idx) => {
                        if let HeapData::String(s) = &vm_ctx.get_heap_mut().objects[*idx].data {
                            s.clone()
                        } else {
                            return Err("Argumen harus berupa string".to_string());
                        }
                    }
                    _ => return Err("Argumen harus berupa string".to_string()),
                };

                unsafe {
                    match Library::new(&path) {
                        Ok(lib) => {
                            let idx = vm_ctx
                                .get_heap_mut()
                                .alloc(HeapData::FfiLibrary(Arc::new(lib)));
                            let mut lib_obj = HashMap::new();
                            lib_obj.insert("id".to_string(), Value::Angka(idx as f64));
                            let obj_idx = vm_ctx.get_heap_mut().alloc(HeapData::Kamus(lib_obj));
                            Ok(Value::Kamus(obj_idx))
                        }
                        Err(e) => Err(format!("Gagal memuat library {}: {}", path, e)),
                    }
                }
            }),
        };
        let muat_lib_idx = vm.heap.alloc(HeapData::FungsiBawaan(muat_lib_obj));
        modul.insert("muat_lib".to_string(), Value::FungsiBawaan(muat_lib_idx));

        let definisi_obj = FungsiBawaanVM {
            nama: "definisi".to_string(),
            func: Arc::new(|vm_ctx: &mut dyn VmContext, args: Vec<Value>| {
                if args.len() != 4 {
                    return Err("definisi memerlukan 4 argumen: (lib, nama_fungsi, [tipe_arg], tipe_kembalian)".to_string());
                }

                // 1. Dapatkan Library
                let lib_arc = match &args[0] {
                    Value::Kamus(idx) => {
                        let id_val =
                            if let HeapData::Kamus(k) = &vm_ctx.get_heap_mut().objects[*idx].data {
                                k.get("id").cloned()
                            } else {
                                None
                            };
                        match id_val {
                            Some(Value::Angka(n)) => {
                                if let HeapData::FfiLibrary(l) =
                                    &vm_ctx.get_heap_mut().objects[n as usize].data
                                {
                                    l.clone()
                                } else {
                                    return Err("Objek library tidak valid".to_string());
                                }
                            }
                            _ => return Err("Objek library tidak valid".to_string()),
                        }
                    }
                    _ => return Err("Argumen pertama harus berupa objek library".to_string()),
                };

                // 2. Dapatkan nama fungsi
                let nama_fungsi = match &args[1] {
                    Value::String(idx) => {
                        if let HeapData::String(s) = &vm_ctx.get_heap_mut().objects[*idx].data {
                            s.clone()
                        } else {
                            return Err("Nama fungsi harus string".to_string());
                        }
                    }
                    _ => return Err("Nama fungsi harus string".to_string()),
                };

                // 3. Dapatkan tipe argumen
                let mut arg_types_str = Vec::new();
                match &args[2] {
                    Value::Array(idx) => {
                        let arr_clone = if let HeapData::Array(arr) =
                            &vm_ctx.get_heap_mut().objects[*idx].data
                        {
                            arr.clone()
                        } else {
                            return Err("Tipe argumen harus array".to_string());
                        };

                        for item in arr_clone {
                            if let Value::String(s_idx) = item {
                                if let HeapData::String(s) =
                                    &vm_ctx.get_heap_mut().objects[s_idx].data
                                {
                                    arg_types_str.push(s.clone());
                                }
                            } else {
                                return Err("Tipe argumen harus array of string".to_string());
                            }
                        }
                    }
                    _ => return Err("Argumen ketiga harus berupa array".to_string()),
                }

                // 4. Dapatkan tipe kembalian
                let ret_type_str = match &args[3] {
                    Value::String(idx) => {
                        if let HeapData::String(s) = &vm_ctx.get_heap_mut().objects[*idx].data {
                            s.clone()
                        } else {
                            return Err("Tipe kembalian harus string".to_string());
                        }
                    }
                    _ => return Err("Tipe kembalian harus string".to_string()),
                };

                // Kembalikan FungsiBawaan yang akan menangani pemanggilan
                let dynamic_call_obj = FungsiBawaanVM {
                    nama: nama_fungsi.clone(),
                    func: Arc::new(move |call_ctx: &mut dyn VmContext, call_args: Vec<Value>| {
                        if call_args.len() != arg_types_str.len() {
                            return Err(format!(
                                "Fungsi {} mengharapkan {} argumen",
                                nama_fungsi,
                                arg_types_str.len()
                            ));
                        }

                        unsafe {
                            let sym: Symbol<unsafe extern "C" fn()> =
                                match lib_arc.get(nama_fungsi.as_bytes()) {
                                    Ok(s) => s,
                                    Err(_) => {
                                        return Err(format!(
                                            "Simbol fungsi {} tidak ditemukan di library",
                                            nama_fungsi
                                        ));
                                    }
                                };

                            let code_ptr = libffi::middle::CodePtr(*sym as *mut _);

                            // Siapkan tipe ffi
                            let mut ffi_arg_types = Vec::new();
                            for t in &arg_types_str {
                                ffi_arg_types.push(str_to_ffi_type(t)?);
                            }
                            let ffi_ret_type = str_to_ffi_type(&ret_type_str)?;

                            let cif =
                                libffi::middle::Cif::new(ffi_arg_types.into_iter(), ffi_ret_type);

                            let mut c_strings = Vec::new();
                            let mut int_vals = Vec::new();
                            let mut float_vals = Vec::new();

                            for (i, t) in arg_types_str.iter().enumerate() {
                                match t.as_str() {
                                    "int" => {
                                        let v = match &call_args[i] {
                                            Value::Angka(n) => *n as i32,
                                            _ => {
                                                return Err(format!(
                                                    "Argumen ke-{} harus angka (int)",
                                                    i + 1
                                                ));
                                            }
                                        };
                                        int_vals.push((i, v));
                                    }
                                    "float" => {
                                        let v = match &call_args[i] {
                                            Value::Angka(n) => *n as f32,
                                            _ => {
                                                return Err(format!(
                                                    "Argumen ke-{} harus angka (float)",
                                                    i + 1
                                                ));
                                            }
                                        };
                                        float_vals.push((i, v));
                                    }
                                    "string" => {
                                        let s = match &call_args[i] {
                                            Value::String(idx) => {
                                                if let HeapData::String(s_val) =
                                                    &call_ctx.get_heap_mut().objects[*idx].data
                                                {
                                                    s_val.clone()
                                                } else {
                                                    return Err(format!(
                                                        "Argumen ke-{} harus string",
                                                        i + 1
                                                    ));
                                                }
                                            }
                                            _ => {
                                                return Err(format!(
                                                    "Argumen ke-{} harus string",
                                                    i + 1
                                                ));
                                            }
                                        };
                                        let c_str = CString::new(s).unwrap();
                                        c_strings.push((i, c_str));
                                    }
                                    "pointer" => {
                                        let v = match &call_args[i] {
                                            Value::Angka(n) => *n as usize,
                                            Value::Kosong => 0,
                                            _ => {
                                                return Err(format!(
                                                    "Argumen pointer harus angka memori"
                                                ));
                                            }
                                        };
                                        int_vals.push((i, v as i32));
                                    }
                                    _ => return Err(format!("Tipe argumen {} tidak didukung", t)),
                                }
                            }

                            let mut ffi_args = Vec::with_capacity(call_args.len());
                            let mut vals_i32 = vec![0i32; call_args.len()];
                            let mut vals_f32 = vec![0f32; call_args.len()];
                            let mut vals_ptr = vec![std::ptr::null::<std::ffi::c_char>(); call_args.len()];

                            for (i, v) in &int_vals {
                                vals_i32[*i] = *v;
                            }
                            for (i, v) in &float_vals {
                                vals_f32[*i] = *v;
                            }
                            for (i, v) in &c_strings {
                                vals_ptr[*i] = v.as_ptr();
                            }

                            for (i, t) in arg_types_str.iter().enumerate() {
                                match t.as_str() {
                                    "int" => ffi_args.push(libffi::middle::Arg::new(&vals_i32[i])),
                                    "float" => {
                                        ffi_args.push(libffi::middle::Arg::new(&vals_f32[i]))
                                    }
                                    "string" => {
                                        ffi_args.push(libffi::middle::Arg::new(&vals_ptr[i]))
                                    }
                                    "pointer" => {
                                        ffi_args.push(libffi::middle::Arg::new(&vals_ptr[i]))
                                    }
                                    _ => unreachable!(),
                                }
                            }

                            match ret_type_str.as_str() {
                                "int" => {
                                    let res: i32 = cif.call(code_ptr, &ffi_args);
                                    Ok(Value::Angka(res as f64))
                                }
                                "float" => {
                                    let res: f32 = cif.call(code_ptr, &ffi_args);
                                    Ok(Value::Angka(res as f64))
                                }
                                "string" => {
                                    let res: *const std::ffi::c_char = cif.call(code_ptr, &ffi_args);
                                    if res.is_null() {
                                        Ok(Value::Kosong)
                                    } else {
                                        let c_str = std::ffi::CStr::from_ptr(res);
                                        let s_idx = call_ctx.get_heap_mut().alloc(
                                            HeapData::String(c_str.to_string_lossy().into_owned()),
                                        );
                                        Ok(Value::String(s_idx))
                                    }
                                }
                                "void" => {
                                    cif.call::<()>(code_ptr, &ffi_args);
                                    Ok(Value::Kosong)
                                }
                                _ => Err(format!("Tipe kembalian {} tidak didukung", ret_type_str)),
                            }
                        }
                    }),
                };
                let dynamic_call_idx = vm_ctx
                    .get_heap_mut()
                    .alloc(HeapData::FungsiBawaan(dynamic_call_obj));
                Ok(Value::FungsiBawaan(dynamic_call_idx))
            }),
        };
        let definisi_idx = vm.heap.alloc(HeapData::FungsiBawaan(definisi_obj));
        modul.insert("definisi".to_string(), Value::FungsiBawaan(definisi_idx));
    }

    #[cfg(not(feature = "enterprise"))]
    {
        let _ = vm; // avoid unused warning
    }

    let mod_idx = vm.heap.alloc(HeapData::Modul(modul));
    vm.set_global("ffi".to_string(), Value::Modul(mod_idx));
}

#[cfg(feature = "enterprise")]
fn str_to_ffi_type(t: &str) -> Result<libffi::middle::Type, String> {
    match t {
        "int" => Ok(libffi::middle::Type::i32()),
        "float" => Ok(libffi::middle::Type::f32()),
        "string" | "pointer" => Ok(libffi::middle::Type::pointer()),
        "void" => Ok(libffi::middle::Type::void()),
        _ => Err(format!("Tipe FFI tidak dikenal: {}", t)),
    }
}

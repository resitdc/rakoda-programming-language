use crate::heap::HeapData;
use crate::machine::VM;
use crate::stdlib::adapter;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    let fungsi_konfigurasi = FungsiBawaanVM {
        nama: "konfigurasi".to_string(),
        func: std::sync::Arc::new(
            move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
                if args.is_empty() {
                    return Err("email.konfigurasi membutuhkan 1 argumen: kamus konfigurasi".to_string());
                }

                // Ambil config Value (Kamus) dan simpan untuk di-closure
                let config_val = args[0];

                let fungsi_kirim = FungsiBawaanVM {
                    nama: "kirim".to_string(),
                    func: std::sync::Arc::new(
                        move |ctx2: &mut dyn VmContext, mut kirim_args: Vec<Value>| -> Result<Value, String> {
                            if kirim_args.is_empty() {
                                return Err("transporter.kirim membutuhkan 1 argumen: kamus pesan".to_string());
                            }

                            // 1. Dapatkan config
                            let heap = ctx2.get_heap_mut();
                            let config_nilai = adapter::value_ke_nilai(&config_val, heap);
                            let config_kamus = match config_nilai {
                                stdlib::jenis::NilaiRpl::Kamus(k) => k,
                                _ => return Err("Konfigurasi transporter tidak valid".to_string()),
                            };

                            // 2. Resolve attachment paths jika ada
                            if let Value::Kamus(k_idx) = kirim_args[0] {
                                let lampiran_array_idx = {
                                    let heap = ctx2.get_heap_mut();
                                    heap.get_kamus(k_idx).get("lampiran").copied()
                                };
                                
                                if let Some(Value::Array(a_idx)) = lampiran_array_idx {
                                    let arr_len = {
                                        let heap = ctx2.get_heap_mut();
                                        heap.get_array(a_idx).len()
                                    };
                                    for i in 0..arr_len {
                                        let item_val = {
                                            let heap = ctx2.get_heap_mut();
                                            heap.get_array(a_idx)[i]
                                        };
                                        if let Value::Kamus(lamp_k_idx) = item_val {
                                            let path_s_idx = {
                                                let heap = ctx2.get_heap_mut();
                                                heap.get_kamus(lamp_k_idx).get("path").copied()
                                            };
                                            if let Some(Value::String(idx)) = path_s_idx {
                                                let p = {
                                                    let heap = ctx2.get_heap_mut();
                                                    heap.get_string(idx).clone()
                                                };
                                                let p_resolved = adapter::resolve_path(ctx2, &p);
                                                let new_s_idx = {
                                                    let heap = ctx2.get_heap_mut();
                                                    heap.alloc(HeapData::String(p_resolved))
                                                };
                                                {
                                                    let heap = ctx2.get_heap_mut();
                                                    heap.get_kamus_mut(lamp_k_idx).insert("path".to_string(), Value::String(new_s_idx));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // 3. Konversi argumen pesan ke NilaiRpl
                            let pesan_nilai = adapter::value_ke_nilai(&kirim_args[0], ctx2.get_heap_mut());
                            let pesan_kamus = match pesan_nilai {
                                stdlib::jenis::NilaiRpl::Kamus(k) => k,
                                _ => return Err("transporter.kirim membutuhkan argumen berupa kamus pesan".to_string()),
                            };

                            // 4. Kirim via stdlib murni
                            match stdlib::email::kirim_impl(&config_kamus, &pesan_kamus) {
                                Ok(res) => Ok(adapter::nilai_ke_value(&res, ctx2.get_heap_mut())),
                                Err(e) => Err(e),
                            }
                        },
                    ),
                };

                let kirim_idx = ctx.get_heap_mut().alloc(HeapData::FungsiBawaan(fungsi_kirim));
                
                let mut trans_dict = HashMap::new();
                trans_dict.insert("kirim".to_string(), Value::FungsiBawaan(kirim_idx));
                let dict_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(trans_dict));
                Ok(Value::Kamus(dict_idx))
            },
        ),
    };

    let idx = vm.heap.alloc(HeapData::FungsiBawaan(fungsi_konfigurasi));
    module_dict.insert("konfigurasi".to_string(), Value::FungsiBawaan(idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("email".to_string(), Value::Kamus(dict_idx));
}

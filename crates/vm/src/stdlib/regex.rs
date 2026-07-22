use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    fn buat_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("regex.buat membutuhkan 1 argumen: pola regex".to_string());
        }
        
        let pola = args[0].to_string(ctx.get_heap_mut());
        let re = Regex::new(&pola).map_err(|e| format!("Pola regex tidak valid: {}", e))?;
        
        let re_arc = Arc::new(re);
        let mut obj_dict = HashMap::new();
        
        // method cocokan
        let re_cocokan = re_arc.clone();
        let cocokan_wrapper = move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
            if args.is_empty() {
                return Err("cocokan membutuhkan 1 argumen teks".to_string());
            }
            let teks = args[0].to_string(ctx.get_heap_mut());
            Ok(Value::Boolean(re_cocokan.is_match(&teks)))
        };
        let cocokan_func = FungsiBawaanVM {
            nama: "cocokan".to_string(),
            func: Arc::new(cocokan_wrapper),
        };
        let cocokan_idx = ctx.get_heap_mut().alloc(HeapData::FungsiBawaan(cocokan_func));
        obj_dict.insert("cocokan".to_string(), Value::FungsiBawaan(cocokan_idx));
        
        // method cari
        let re_cari = re_arc.clone();
        let cari_wrapper = move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
            if args.is_empty() {
                return Err("cari membutuhkan 1 argumen teks".to_string());
            }
            let teks = args[0].to_string(ctx.get_heap_mut());
            if let Some(mat) = re_cari.find(&teks) {
                let match_str = mat.as_str().to_string();
                let str_idx = ctx.get_heap_mut().alloc(HeapData::String(match_str));
                Ok(Value::String(str_idx))
            } else {
                let empty_idx = ctx.get_heap_mut().alloc(HeapData::String("".to_string()));
                Ok(Value::String(empty_idx))
            }
        };
        let cari_func = FungsiBawaanVM {
            nama: "cari".to_string(),
            func: Arc::new(cari_wrapper),
        };
        let cari_idx = ctx.get_heap_mut().alloc(HeapData::FungsiBawaan(cari_func));
        obj_dict.insert("cari".to_string(), Value::FungsiBawaan(cari_idx));
        
        // method ganti
        let re_ganti = re_arc.clone();
        let ganti_wrapper = move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
            if args.len() < 2 {
                return Err("ganti membutuhkan 2 argumen: teks, teks_pengganti".to_string());
            }
            let teks = args[0].to_string(ctx.get_heap_mut());
            let pengganti = args[1].to_string(ctx.get_heap_mut());
            
            let hasil = re_ganti.replace_all(&teks, &pengganti).to_string();
            let hasil_idx = ctx.get_heap_mut().alloc(HeapData::String(hasil));
            Ok(Value::String(hasil_idx))
        };
        let ganti_func = FungsiBawaanVM {
            nama: "ganti".to_string(),
            func: Arc::new(ganti_wrapper),
        };
        let ganti_idx = ctx.get_heap_mut().alloc(HeapData::FungsiBawaan(ganti_func));
        obj_dict.insert("ganti".to_string(), Value::FungsiBawaan(ganti_idx));
        
        let obj_idx = ctx.get_heap_mut().alloc(HeapData::Kamus(obj_dict));
        Ok(Value::Kamus(obj_idx))
    }

    let buat_func = FungsiBawaanVM {
        nama: "buat".to_string(),
        func: std::sync::Arc::new(buat_wrapper),
    };
    let buat_idx = vm.heap.alloc(HeapData::FungsiBawaan(buat_func));
    module_dict.insert("buat".to_string(), Value::FungsiBawaan(buat_idx));

    let module_idx = vm.heap.alloc(HeapData::Modul(module_dict));
    vm.set_global("regex".to_string(), Value::Modul(module_idx));
}

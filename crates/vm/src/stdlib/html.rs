use crate::heap::HeapData;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::collections::HashMap;

#[cfg(feature = "enterprise")]
use std::sync::Arc;
#[cfg(feature = "enterprise")]
use scraper::{Html, Selector, ElementRef};

#[cfg(feature = "enterprise")]
pub struct SafeHtml(pub scraper::Html);

#[cfg(feature = "enterprise")]
unsafe impl Send for SafeHtml {}

#[cfg(feature = "enterprise")]
unsafe impl Sync for SafeHtml {}

pub fn register(machine: &mut crate::VM) {
    let mut html_module = HashMap::new();

    #[cfg(feature = "enterprise")]
    {
        let parse_func = FungsiBawaanVM {
            nama: "parse".to_string(),
            func: Arc::new(|vm: &mut dyn VmContext, args: Vec<Value>| {
                if args.is_empty() {
                    return Err("Fungsi parse membutuhkan argumen string HTML".to_string());
                }
                let html_str = if let Value::String(idx) = &args[0] {
                    vm.get_heap_mut().get_string(*idx).clone()
                } else {
                    return Err("Argumen HTML harus berupa string".to_string());
                };

                let document = Arc::new(SafeHtml(Html::parse_document(&html_str)));
                let root_id = document.0.tree.root().id();

                Ok(buat_elemen_kamus(vm, document, root_id))
            }),
        };
        
        let parse_idx = machine.heap.alloc(HeapData::FungsiBawaan(parse_func));
        html_module.insert("parse".to_string(), Value::FungsiBawaan(parse_idx));
    }

    #[cfg(not(feature = "enterprise"))]
    {
        let dummy_func = FungsiBawaanVM {
            nama: "parse".to_string(),
            func: std::sync::Arc::new(|_vm: &mut dyn VmContext, _args: Vec<Value>| {
                Err("Modul HTML tidak tersedia (membutuhkan fitur enterprise)".to_string())
            }),
        };
        let parse_idx = machine.heap.alloc(HeapData::FungsiBawaan(dummy_func));
        html_module.insert("parse".to_string(), Value::FungsiBawaan(parse_idx));
    }

    let mod_idx = machine.heap.alloc(HeapData::Modul(html_module));
    machine.set_global("html".to_string(), Value::Modul(mod_idx));
}

#[cfg(feature = "enterprise")]
fn buat_elemen_kamus(vm: &mut dyn VmContext, document: Arc<SafeHtml>, node_id: ego_tree::NodeId) -> Value {
    let mut kamus = HashMap::new();

    // querySelector
    let doc_qs = document.clone();
    let node_id_qs = node_id;
    let qs_func = FungsiBawaanVM {
        nama: "querySelector".to_string(),
        func: Arc::new(move |vm_ctx: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("querySelector butuh argumen selector".to_string());
            }
            let sel_str = if let Value::String(idx) = &args[0] {
                vm_ctx.get_heap_mut().get_string(*idx).clone()
            } else {
                return Err("Selector harus berupa string".to_string());
            };

            let selector = Selector::parse(&sel_str).map_err(|e| format!("Selector tidak valid: {:?}", e))?;
            let node = doc_qs.0.tree.get(node_id_qs).unwrap();

            let found = if let Some(el_ref) = ElementRef::wrap(node) {
                el_ref.select(&selector).next().map(|e| e.id())
            } else if node_id_qs == doc_qs.0.tree.root().id() {
                doc_qs.0.select(&selector).next().map(|e| e.id())
            } else {
                None
            };

            if let Some(found_id) = found {
                Ok(buat_elemen_kamus(vm_ctx, doc_qs.clone(), found_id))
            } else {
                Ok(Value::Kosong)
            }
        }),
    };
    let qs_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(qs_func));
    kamus.insert("querySelector".to_string(), Value::FungsiBawaan(qs_idx));

    // querySelectorAll
    let doc_qsa = document.clone();
    let node_id_qsa = node_id;
    let qsa_func = FungsiBawaanVM {
        nama: "querySelectorAll".to_string(),
        func: Arc::new(move |vm_ctx: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("querySelectorAll butuh argumen selector".to_string());
            }
            let sel_str = if let Value::String(idx) = &args[0] {
                vm_ctx.get_heap_mut().get_string(*idx).clone()
            } else {
                return Err("Selector harus berupa string".to_string());
            };

            let selector = Selector::parse(&sel_str).map_err(|e| format!("Selector tidak valid: {:?}", e))?;
            let node = doc_qsa.0.tree.get(node_id_qsa).unwrap();
            
            let mut arr = Vec::new();

            if let Some(el_ref) = ElementRef::wrap(node) {
                for found in el_ref.select(&selector) {
                    arr.push(buat_elemen_kamus(vm_ctx, doc_qsa.clone(), found.id()));
                }
            } else if node_id_qsa == doc_qsa.0.tree.root().id() {
                for found in doc_qsa.0.select(&selector) {
                    arr.push(buat_elemen_kamus(vm_ctx, doc_qsa.clone(), found.id()));
                }
            }

            let arr_idx = vm_ctx.get_heap_mut().alloc(HeapData::Array(arr));
            Ok(Value::Array(arr_idx))
        }),
    };
    let qsa_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(qsa_func));
    kamus.insert("querySelectorAll".to_string(), Value::FungsiBawaan(qsa_idx));

    // teks()
    let doc_teks = document.clone();
    let node_id_teks = node_id;
    let teks_func = FungsiBawaanVM {
        nama: "teks".to_string(),
        func: Arc::new(move |vm_ctx: &mut dyn VmContext, _args: Vec<Value>| {
            let node = doc_teks.0.tree.get(node_id_teks).unwrap();
            let mut text = String::new();
            if let Some(el_ref) = ElementRef::wrap(node) {
                text = el_ref.text().collect::<Vec<_>>().join(" ");
            }
            let s_idx = vm_ctx.get_heap_mut().alloc(HeapData::String(text));
            Ok(Value::String(s_idx))
        }),
    };
    let teks_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(teks_func));
    kamus.insert("teks".to_string(), Value::FungsiBawaan(teks_idx));

    // html()
    let doc_html = document.clone();
    let node_id_html = node_id;
    let html_func = FungsiBawaanVM {
        nama: "html".to_string(),
        func: Arc::new(move |vm_ctx: &mut dyn VmContext, _args: Vec<Value>| {
            let node = doc_html.0.tree.get(node_id_html).unwrap();
            let mut html_str = String::new();
            if let Some(el_ref) = ElementRef::wrap(node) {
                html_str = el_ref.html();
            }
            let s_idx = vm_ctx.get_heap_mut().alloc(HeapData::String(html_str));
            Ok(Value::String(s_idx))
        }),
    };
    let html_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(html_func));
    kamus.insert("html".to_string(), Value::FungsiBawaan(html_idx));

    // atribut(nama)
    let doc_attr = document.clone();
    let node_id_attr = node_id;
    let attr_func = FungsiBawaanVM {
        nama: "atribut".to_string(),
        func: Arc::new(move |vm_ctx: &mut dyn VmContext, args: Vec<Value>| {
            if args.is_empty() {
                return Err("atribut butuh argumen nama atribut".to_string());
            }
            let nama_attr = if let Value::String(idx) = &args[0] {
                vm_ctx.get_heap_mut().get_string(*idx).clone()
            } else {
                return Err("Nama atribut harus berupa string".to_string());
            };

            let node = doc_attr.0.tree.get(node_id_attr).unwrap();
            if let Some(el_ref) = ElementRef::wrap(node) {
                if let Some(val) = el_ref.value().attr(&nama_attr) {
                    let s_idx = vm_ctx.get_heap_mut().alloc(HeapData::String(val.to_string()));
                    return Ok(Value::String(s_idx));
                }
            }
            Ok(Value::Kosong)
        }),
    };
    let attr_idx = vm.get_heap_mut().alloc(HeapData::FungsiBawaan(attr_func));
    kamus.insert("atribut".to_string(), Value::FungsiBawaan(attr_idx));

    let k_idx = vm.get_heap_mut().alloc(HeapData::Kamus(kamus));
    Value::Kamus(k_idx)
}

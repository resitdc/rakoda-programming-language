use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use chrono::Local;
use std::collections::HashMap;

fn log_impl(
    ctx: &mut dyn VmContext,
    args: Vec<Value>,
    level: &str,
    color: &str,
) -> Result<Value, String> {
    if args.is_empty() {
        return Err(format!(
            "Fungsi 'log.{}' membutuhkan setidaknya 1 argumen",
            level
        ));
    }

    let mut msgs = Vec::new();
    for arg in args {
        msgs.push(arg.to_string(ctx.get_heap_mut()));
    }
    let combined_msg = msgs.join(" ");

    let time_str = Local::now().format("%Y-%m-%d %H:%M:%S.%3f").to_string();

    let func_name = ctx.current_function_info().0;
    let mut location_info = format!("{}:?", func_name);

    if let Some(loc) = ctx.current_lokasi() {
        location_info = format!("{}:{}", func_name, loc.baris);
    }

    println!(
        "{}{:<23} | {:<5} | [{}] | {}\x1b[0m",
        color,
        time_str,
        level.to_uppercase(),
        location_info,
        combined_msg
    );

    super::dev_dashboard::record_log(super::dev_dashboard::LogTelemetry {
        timestamp: time_str,
        level: level.to_string(),
        message: combined_msg,
        caller: location_info,
    });

    Ok(Value::Kosong)
}

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    // info
    let info_func = FungsiBawaanVM {
        nama: "info".to_string(),
        func: |ctx, args| log_impl(ctx, args, "info", "\x1b[36m"), // cyan
    };
    let info_idx = vm.heap.alloc(HeapData::FungsiBawaan(info_func));
    module_dict.insert("info".to_string(), Value::FungsiBawaan(info_idx));

    // warning
    let warning_func = FungsiBawaanVM {
        nama: "peringatan".to_string(),
        func: |ctx, args| log_impl(ctx, args, "warn", "\x1b[33m"), // yellow
    };
    let warning_idx = vm.heap.alloc(HeapData::FungsiBawaan(warning_func));
    module_dict.insert("peringatan".to_string(), Value::FungsiBawaan(warning_idx));
    module_dict.insert("warning".to_string(), Value::FungsiBawaan(warning_idx)); // alias

    // error
    let error_func = FungsiBawaanVM {
        nama: "salah".to_string(),
        func: |ctx, args| log_impl(ctx, args, "error", "\x1b[31m"), // red
    };
    let error_idx = vm.heap.alloc(HeapData::FungsiBawaan(error_func));
    module_dict.insert("salah".to_string(), Value::FungsiBawaan(error_idx));
    module_dict.insert("error".to_string(), Value::FungsiBawaan(error_idx)); // alias

    // debug
    let debug_func = FungsiBawaanVM {
        nama: "debug".to_string(),
        func: |ctx, args| log_impl(ctx, args, "debug", "\x1b[35m"), // magenta
    };
    let debug_idx = vm.heap.alloc(HeapData::FungsiBawaan(debug_func));
    module_dict.insert("debug".to_string(), Value::FungsiBawaan(debug_idx));

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("log".to_string(), Value::Kamus(dict_idx));
}

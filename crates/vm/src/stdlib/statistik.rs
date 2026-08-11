use crate::heap::{Heap, HeapData};
use crate::value::{Value, FungsiBawaanVM, VmContext};
use std::collections::HashMap;
use std::sync::Arc;
use rand_mt::Mt64;
use std::cell::RefCell;

pub fn register(vm: &mut crate::machine::VM) {
    let mut methods = HashMap::new();

    let funcs = vec![
        ("mean", statistik_mean as fn(&mut dyn VmContext, Vec<Value>) -> Result<Value, String>),
        ("median", statistik_median),
        ("variance", statistik_variance),
        ("std", statistik_std),
        ("min", statistik_min),
        ("max", statistik_max),
        ("sum", statistik_sum),
        ("argmax", statistik_argmax),
        ("buatGenerator", statistik_buat_generator),
        ("angkaAcak", statistik_angka_acak),
    ];

    for (name, func) in funcs {
        let func_obj = FungsiBawaanVM {
            nama: format!("Statistik.{}", name),
            func: Arc::new(func),
        };
        let idx = vm.heap.alloc(HeapData::FungsiBawaan(func_obj));
        methods.insert(name.to_string(), Value::FungsiBawaan(idx));
    }

    let modul_idx = vm.heap.alloc(HeapData::Modul(methods));
    vm.set_global("Statistik".to_string(), Value::Modul(modul_idx));
}

fn get_tensor_data(heap: &Heap, args: &[Value], fn_name: &str) -> Result<Vec<f64>, String> {
    if args.is_empty() {
        return Err(format!("{} membutuhkan 1 argumen", fn_name));
    }

    match &args[0] {
        Value::Float64Array(idx) => {
            let tensor = heap.get_f64_tensor(*idx);
            Ok(tensor.data.read().unwrap().clone())
        }
        Value::Int32Array(idx) => {
            let tensor = heap.get_i32_tensor(*idx);
            let data = tensor.data.read().unwrap();
            Ok(data.iter().map(|&x| x as f64).collect())
        }
        Value::Array(idx) => {
            let arr = heap.get_array(*idx);
            let mut data = Vec::with_capacity(arr.len());
            for v in arr {
                if let Value::Angka(n) = v {
                    data.push(*n);
                } else {
                    return Err(format!("{} hanya mendukung array angka", fn_name));
                }
            }
            Ok(data)
        }
        _ => Err(format!("Argumen pertama {} harus berupa Float64Array, Int32Array, atau Array angka", fn_name)),
    }
}

fn statistik_mean(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let data = get_tensor_data(heap, &args, "Statistik.mean")?;
    if data.is_empty() {
        return Err("Array kosong tidak memiliki nilai rata-rata".to_string());
    }
    let sum: f64 = data.iter().sum();
    Ok(Value::Angka(sum / data.len() as f64))
}

fn statistik_median(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let mut data = get_tensor_data(heap, &args, "Statistik.median")?;
    if data.is_empty() {
        return Err("Array kosong tidak memiliki median".to_string());
    }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = data.len() / 2;
    if data.len() % 2 == 0 {
        Ok(Value::Angka((data[mid - 1] + data[mid]) / 2.0))
    } else {
        Ok(Value::Angka(data[mid]))
    }
}

fn statistik_variance(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let data = get_tensor_data(heap, &args, "Statistik.variance")?;
    if data.len() < 2 {
        return Err("Variance membutuhkan minimal 2 elemen".to_string());
    }
    let sum: f64 = data.iter().sum();
    let mean = sum / data.len() as f64;
    let variance: f64 = data.iter().map(|value| {
        let diff = mean - *value;
        diff * diff
    }).sum::<f64>() / (data.len() - 1) as f64;
    Ok(Value::Angka(variance))
}

fn statistik_std(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let var_val = statistik_variance(ctx, args)?;
    if let Value::Angka(var) = var_val {
        Ok(Value::Angka(var.sqrt()))
    } else {
        unreachable!()
    }
}

fn statistik_sum(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let data = get_tensor_data(heap, &args, "Statistik.sum")?;
    let sum: f64 = data.iter().sum();
    Ok(Value::Angka(sum))
}

fn statistik_min(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let data = get_tensor_data(heap, &args, "Statistik.min")?;
    let min = data.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if let Some(&val) = min {
        Ok(Value::Angka(val))
    } else {
        Err("Array kosong".to_string())
    }
}

fn statistik_max(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let data = get_tensor_data(heap, &args, "Statistik.max")?;
    let max = data.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if let Some(&val) = max {
        Ok(Value::Angka(val))
    } else {
        Err("Array kosong".to_string())
    }
}

fn statistik_argmax(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    let heap = ctx.get_heap_mut();
    let data = get_tensor_data(heap, &args, "Statistik.argmax")?;
    if data.is_empty() {
        return Err("Array kosong".to_string());
    }
    let mut max_idx = 0;
    let mut max_val = data[0];
    for (i, &val) in data.iter().enumerate().skip(1) {
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }
    Ok(Value::Angka(max_idx as f64))
}

fn statistik_buat_generator(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Statistik.buatGenerator membutuhkan 1 argumen angka (seed)".to_string());
    }
    if let Value::Angka(seed) = args[0] {
        let rng = Mt64::new(seed as u64);
        let heap = ctx.get_heap_mut();
        let idx = heap.alloc(HeapData::RandomGenerator(RefCell::new(rng)));
        Ok(Value::RandomGenerator(idx))
    } else {
        Err("Seed harus berupa angka".to_string())
    }
}

fn statistik_angka_acak(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("Statistik.angkaAcak membutuhkan 1 argumen RandomGenerator".to_string());
    }
    if let Value::RandomGenerator(idx) = args[0] {
        let heap = ctx.get_heap_mut();
        let mut rng = heap.get_random_generator(idx).borrow_mut();
        let val = rng.next_u64() as f64 / u64::MAX as f64;
        Ok(Value::Angka(val))
    } else {
        Err("Argumen pertama harus berupa RandomGenerator".to_string())
    }
}

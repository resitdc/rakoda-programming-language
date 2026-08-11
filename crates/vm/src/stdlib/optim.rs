use std::collections::HashMap;
use std::sync::Arc;

use crate::heap::HeapData;
use crate::value::{FungsiBawaanVM, Value, VmContext};

pub fn bawaan_optim(vm: &mut crate::machine::VM) {
    let mut optim_mod = HashMap::new();

    // Optim.SGD([tensor1, tensor2], learning_rate)
    let sgd_wrapper = |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
        if args.len() != 2 {
            return Err("Optim.SGD membutuhkan 2 argumen: array tensor dan learning_rate".to_string());
        }

        let tensor_list = if let Value::Array(arr_idx) = &args[0] {
            ctx.get_heap_mut().get_array(*arr_idx).clone()
        } else {
            return Err("Argumen pertama Optim.SGD harus berupa array tensor".to_string());
        };

        let lr = if let Value::Angka(n) = &args[1] {
            *n
        } else {
            return Err("Argumen kedua Optim.SGD harus berupa angka (learning_rate)".to_string());
        };

        // Verifikasi semua elemen di array adalah tensor
        let mut float64_indices = Vec::new();
        for val in &tensor_list {
            if let Value::Float64Array(idx) = val {
                float64_indices.push(*idx);
            } else {
                return Err("Semua elemen di dalam array SGD harus berupa Float64Array".to_string());
            }
        }

        let mut instance_mod = HashMap::new();

        // 1. method step()
        let step_indices = float64_indices.clone();
        let step_func = move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
            if args.len() != 0 {
                return Err("method step() tidak menerima argumen".to_string());
            }
            let heap = ctx.get_heap_mut();
            
            for &idx in &step_indices {
                if let Some(tensor) = heap.get_f64_tensor_opt_mut(idx) {
                    if let Some(grad_arc) = &tensor.grad {
                        let grad = grad_arc.read().unwrap();
                        let mut data = tensor.data.write().unwrap();
                        // W = W - lr * W.grad
                        for i in 0..data.len() {
                            data[i] -= lr * grad[i];
                        }
                    }
                }
            }
            Ok(Value::Kosong)
        };
        
        let step_obj = FungsiBawaanVM {
            nama: "SGD.step".to_string(),
            func: Arc::new(step_func),
        };
        let step_idx = ctx.get_heap_mut().alloc(HeapData::FungsiBawaan(step_obj));
        instance_mod.insert("step".to_string(), Value::FungsiBawaan(step_idx));

        // 2. method zero_grad()
        let zero_indices = float64_indices.clone();
        let zero_func = move |ctx: &mut dyn VmContext, args: Vec<Value>| -> Result<Value, String> {
            if args.len() != 0 {
                return Err("method zero_grad() tidak menerima argumen".to_string());
            }
            let heap = ctx.get_heap_mut();
            
            for &idx in &zero_indices {
                if let Some(tensor) = heap.get_f64_tensor_opt_mut(idx) {
                    if let Some(grad_arc) = &tensor.grad {
                        let mut grad = grad_arc.write().unwrap();
                        for i in 0..grad.len() {
                            grad[i] = 0.0;
                        }
                    }
                }
            }
            Ok(Value::Kosong)
        };
        
        let zero_obj = FungsiBawaanVM {
            nama: "SGD.zero_grad".to_string(),
            func: Arc::new(zero_func),
        };
        let zero_idx = ctx.get_heap_mut().alloc(HeapData::FungsiBawaan(zero_obj));
        instance_mod.insert("zero_grad".to_string(), Value::FungsiBawaan(zero_idx));

        let instance_idx = ctx.get_heap_mut().alloc(HeapData::Modul(instance_mod));
        Ok(Value::Modul(instance_idx))
    };

    let sgd_obj = FungsiBawaanVM {
        nama: "Optim.SGD".to_string(),
        func: Arc::new(sgd_wrapper),
    };
    let sgd_idx = vm.heap.alloc(HeapData::FungsiBawaan(sgd_obj));
    optim_mod.insert("SGD".to_string(), Value::FungsiBawaan(sgd_idx));

    let optim_mod_idx = vm.heap.alloc(HeapData::Modul(optim_mod));
    vm.set_global("Optim".to_string(), Value::Modul(optim_mod_idx));
}

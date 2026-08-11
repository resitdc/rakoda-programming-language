use crate::heap::HeapData;
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use std::sync::{Arc, RwLock};

pub fn register(vm: &mut VM) {
    fn float64_array_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("Float64Array butuh argumen bentuk (shape)".to_string());
        }
        let arg = &args[0];
        let mut shape = Vec::new();
        let heap = ctx.get_heap_mut();

        match arg {
            Value::Array(idx) => {
                let elements = heap.get_array(*idx);
                for el in elements {
                    if let Value::Angka(n) = el {
                        shape.push(*n as usize);
                    } else {
                        return Err("Argumen bentuk (shape) harus berupa array angka".to_string());
                    }
                }
            }
            Value::Angka(n) => {
                shape.push(*n as usize);
            }
            _ => return Err("Argumen harus berupa Array bentuk atau Angka tunggal".to_string()),
        }

        let total_size: usize = shape.iter().product();
        let data = Arc::new(RwLock::new(vec![0.0; total_size]));

        let mut strides = vec![1; shape.len()];
        let mut current_stride = 1;
        for i in (0..shape.len()).rev() {
            strides[i] = current_stride;
            current_stride *= shape[i];
        }

        let tensor = crate::heap::Tensor {
            data,
            shape,
            strides,
            offset: 0,
        };

        let new_idx = heap.alloc(HeapData::Float64Array(tensor));
        Ok(Value::Float64Array(new_idx))
    }

    let float64_array = FungsiBawaanVM {
        nama: "Float64Array".to_string(),
        func: std::sync::Arc::new(float64_array_wrapper),
    };
    let float64_array_idx = vm.heap.alloc(HeapData::FungsiBawaan(float64_array));
    vm.set_global(
        "Float64Array".to_string(),
        Value::FungsiBawaan(float64_array_idx),
    );

    fn int32_array_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.is_empty() {
            return Err("Int32Array butuh argumen bentuk (shape)".to_string());
        }
        let arg = &args[0];
        let mut shape = Vec::new();
        let heap = ctx.get_heap_mut();

        match arg {
            Value::Array(idx) => {
                let elements = heap.get_array(*idx);
                for el in elements {
                    if let Value::Angka(n) = el {
                        shape.push(*n as usize);
                    } else {
                        return Err("Argumen bentuk (shape) harus berupa array angka".to_string());
                    }
                }
            }
            Value::Angka(n) => {
                shape.push(*n as usize);
            }
            _ => return Err("Argumen harus berupa Array bentuk atau Angka tunggal".to_string()),
        }

        let total_size: usize = shape.iter().product();
        let data = Arc::new(RwLock::new(vec![0; total_size]));

        let mut strides = vec![1; shape.len()];
        let mut current_stride = 1;
        for i in (0..shape.len()).rev() {
            strides[i] = current_stride;
            current_stride *= shape[i];
        }

        let tensor = crate::heap::Tensor {
            data,
            shape,
            strides,
            offset: 0,
        };

        let new_idx = heap.alloc(HeapData::Int32Array(tensor));
        Ok(Value::Int32Array(new_idx))
    }

    let int32_array = FungsiBawaanVM {
        nama: "Int32Array".to_string(),
        func: std::sync::Arc::new(int32_array_wrapper),
    };
    let int32_array_idx = vm.heap.alloc(HeapData::FungsiBawaan(int32_array));
    vm.set_global(
        "Int32Array".to_string(),
        Value::FungsiBawaan(int32_array_idx),
    );
}

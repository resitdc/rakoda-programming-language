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

        let mut requires_grad = false;
        if args.len() > 1 {
            if let Value::Boolean(true) = args[1] {
                requires_grad = true;
            }
        }

        let tensor = crate::heap::Tensor {
            data,
            shape,
            strides,
            offset: 0,
            requires_grad,
            grad: None,
            tape_node: None,
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
            requires_grad: false,
            grad: None,
            tape_node: None,
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

    // --- Matriks Module ---
    let mut matriks_mod = std::collections::HashMap::new();

    // Matriks.dot(a, b)
    fn matriks_dot_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err("Matriks.dot butuh 2 argumen".to_string());
        }
        let heap = ctx.get_heap_mut();
        if let (Value::Float64Array(a_idx), Value::Float64Array(b_idx)) = (&args[0], &args[1]) {
            let a_tensor = heap.get_f64_tensor(*a_idx).clone();
            let b_tensor = heap.get_f64_tensor(*b_idx).clone();
            if a_tensor.shape.len() != 2 || b_tensor.shape.len() != 2 {
                return Err("Matriks.dot hanya didukung untuk matriks 2D".to_string());
            }
            if a_tensor.shape[1] != b_tensor.shape[0] {
                return Err(format!("Dimensi tidak valid untuk perkalian: kolom A ({}) != baris B ({})", a_tensor.shape[1], b_tensor.shape[0]));
            }

            // Convert to nalgebra DMatrix (row-major read, column-major internal, but we can use from_row_slice)
            let a_data = a_tensor.data.read().unwrap();
            let b_data = b_tensor.data.read().unwrap();
            
            // Note: from_row_slice requires flat dense data. Since we have contiguous data for now:
            let a_mat = nalgebra::DMatrix::from_row_slice(a_tensor.shape[0], a_tensor.shape[1], &a_data);
            let b_mat = nalgebra::DMatrix::from_row_slice(b_tensor.shape[0], b_tensor.shape[1], &b_data);

            let res_mat = a_mat * b_mat;

            // Extract back to row-major
            let mut out_data = Vec::with_capacity(res_mat.len());
            for r in 0..res_mat.nrows() {
                for c in 0..res_mat.ncols() {
                    out_data.push(res_mat[(r, c)]);
                }
            }

            let new_shape = vec![a_tensor.shape[0], b_tensor.shape[1]];
            let mut out_strides = vec![1; 2];
            out_strides[0] = new_shape[1];
            
            let mut res_tensor = crate::heap::Tensor {
                data: Arc::new(RwLock::new(out_data)),
                shape: new_shape,
                strides: out_strides,
                offset: 0,
                requires_grad: false,
                grad: None,
                tape_node: None,
            };

            if a_tensor.requires_grad || b_tensor.requires_grad {
                res_tensor.requires_grad = true;
                let node = crate::autograd::TapeNode {
                    op: crate::autograd::BackwardOp::Matmul,
                    parents: vec![*a_idx, *b_idx],
                    self_tensor_idx: 0,
                };
                let tape_node_id = heap.tape.push(node);
                res_tensor.tape_node = Some(tape_node_id);
            }

            let new_idx = heap.alloc(HeapData::Float64Array(res_tensor));
            
            if heap.get_f64_tensor(new_idx).requires_grad {
                let tape_node_id = heap.get_f64_tensor(new_idx).tape_node.unwrap();
                heap.tape.nodes[tape_node_id].self_tensor_idx = new_idx;
            }
            Ok(Value::Float64Array(new_idx))
        } else {
            Err("Matriks.dot saat ini hanya mendukung Float64Array".to_string())
        }
    }

    // Matriks.transpose(a)
    fn matriks_transpose_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("Matriks.transpose butuh 1 argumen".to_string());
        }
        let heap = ctx.get_heap_mut();
        if let Value::Float64Array(idx) = &args[0] {
            let tensor = heap.get_f64_tensor(*idx).clone();
            if tensor.shape.len() != 2 {
                return Err("Matriks.transpose hanya didukung untuk matriks 2D".to_string());
            }
            let mut new_shape = tensor.shape.clone();
            new_shape.reverse();
            let mut new_strides = tensor.strides.clone();
            new_strides.reverse();

            let mut new_tensor = crate::heap::Tensor {
                data: tensor.data,
                shape: new_shape,
                strides: new_strides,
                offset: tensor.offset,
                requires_grad: false,
                grad: None,
                tape_node: None,
            };

            if tensor.requires_grad {
                new_tensor.requires_grad = true;
                let node = crate::autograd::TapeNode {
                    op: crate::autograd::BackwardOp::Transpose,
                    parents: vec![*idx],
                    self_tensor_idx: 0,
                };
                let tape_node_id = heap.tape.push(node);
                new_tensor.tape_node = Some(tape_node_id);
            }

            let new_idx = heap.alloc(HeapData::Float64Array(new_tensor));
            
            if heap.get_f64_tensor(new_idx).requires_grad {
                let tape_node_id = heap.get_f64_tensor(new_idx).tape_node.unwrap();
                heap.tape.nodes[tape_node_id].self_tensor_idx = new_idx;
            }

            Ok(Value::Float64Array(new_idx))
        } else {
            Err("Matriks.transpose saat ini hanya mendukung Float64Array".to_string())
        }
    }

    // Matriks.inverse(a)
    fn matriks_inverse_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("Matriks.inverse butuh 1 argumen".to_string());
        }
        let heap = ctx.get_heap_mut();
        if let Value::Float64Array(idx) = &args[0] {
            let tensor = heap.get_f64_tensor(*idx).clone();
            if tensor.shape.len() != 2 || tensor.shape[0] != tensor.shape[1] {
                return Err("Matriks.inverse hanya didukung untuk matriks persegi 2D".to_string());
            }

            let data = tensor.data.read().unwrap();
            let mat = nalgebra::DMatrix::from_row_slice(tensor.shape[0], tensor.shape[1], &data);
            
            match mat.clone().try_inverse() {
                Some(inv_mat) => {
                    let mut out_data = Vec::with_capacity(inv_mat.len());
                    for r in 0..inv_mat.nrows() {
                        for c in 0..inv_mat.ncols() {
                            out_data.push(inv_mat[(r, c)]);
                        }
                    }
                    let out_strides = vec![tensor.shape[1], 1];
                    let new_tensor = crate::heap::Tensor {
                        data: Arc::new(RwLock::new(out_data)),
                        shape: tensor.shape.clone(),
                        strides: out_strides,
                        offset: 0,
            requires_grad: false,
            grad: None,
            tape_node: None,
                    };
                    let new_idx = heap.alloc(HeapData::Float64Array(new_tensor));
                    Ok(Value::Float64Array(new_idx))
                }
                None => Err("Matriks bersifat singular, tidak dapat di-invers".to_string())
            }
        } else {
            Err("Matriks.inverse saat ini hanya mendukung Float64Array".to_string())
        }
    }

    // Matriks.determinant(a)
    fn matriks_determinant_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("Matriks.determinant butuh 1 argumen".to_string());
        }
        let heap = ctx.get_heap_mut();
        if let Value::Float64Array(idx) = &args[0] {
            let tensor = heap.get_f64_tensor(*idx).clone();
            if tensor.shape.len() != 2 || tensor.shape[0] != tensor.shape[1] {
                return Err("Matriks.determinant hanya didukung untuk matriks persegi 2D".to_string());
            }

            let data = tensor.data.read().unwrap();
            let mat = nalgebra::DMatrix::from_row_slice(tensor.shape[0], tensor.shape[1], &data);
            
            Ok(Value::Angka(mat.determinant()))
        } else {
            Err("Matriks.determinant saat ini hanya mendukung Float64Array".to_string())
        }
    }

    // Matriks.eigen(a)
    fn matriks_eigen_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("Matriks.eigen butuh 1 argumen".to_string());
        }
        let heap = ctx.get_heap_mut();
        if let Value::Float64Array(idx) = &args[0] {
            let tensor = heap.get_f64_tensor(*idx).clone();
            if tensor.shape.len() != 2 || tensor.shape[0] != tensor.shape[1] {
                return Err("Matriks.eigen hanya didukung untuk matriks persegi 2D".to_string());
            }

            let data = tensor.data.read().unwrap();
            let mat = nalgebra::DMatrix::from_row_slice(tensor.shape[0], tensor.shape[1], &data);
            
            // Kami menggunakan SymmetricEigen untuk simplisitas
            // Sebagian besar pengolahan matriks korelasi adalah simetris
            let eigen = nalgebra::SymmetricEigen::new(mat);

            // eigenvalues (1D array)
            let mut val_data = Vec::with_capacity(tensor.shape[0]);
            for i in 0..tensor.shape[0] {
                val_data.push(eigen.eigenvalues[i]);
            }
            let val_tensor = crate::heap::Tensor {
                data: Arc::new(RwLock::new(val_data)),
                shape: vec![tensor.shape[0]],
                strides: vec![1],
                offset: 0,
            requires_grad: false,
            grad: None,
            tape_node: None,
            };
            let val_idx = heap.alloc(HeapData::Float64Array(val_tensor));

            // eigenvectors (2D matrix)
            let mut vec_data = Vec::with_capacity(tensor.shape[0] * tensor.shape[1]);
            for r in 0..tensor.shape[0] {
                for c in 0..tensor.shape[1] {
                    vec_data.push(eigen.eigenvectors[(r, c)]);
                }
            }
            let vec_tensor = crate::heap::Tensor {
                data: Arc::new(RwLock::new(vec_data)),
                shape: tensor.shape.clone(),
                strides: tensor.strides.clone(),
                offset: 0,
            requires_grad: false,
            grad: None,
            tape_node: None,
            };
            let vec_idx = heap.alloc(HeapData::Float64Array(vec_tensor));

            let res_idx = heap.alloc(HeapData::Array(vec![
                Value::Float64Array(val_idx),
                Value::Float64Array(vec_idx),
            ]));
            
            Ok(Value::Array(res_idx))
        } else {
            Err("Matriks.eigen saat ini hanya mendukung Float64Array".to_string())
        }
    }

    fn matriks_grad_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("Matriks.grad membutuhkan 1 argumen".to_string());
        }
        if let Value::Float64Array(idx) = args[0] {
            let heap = ctx.get_heap_mut();
            let tensor = heap.get_f64_tensor(idx);
            if !tensor.requires_grad {
                return Err("Tensor ini tidak memiliki requires_grad = benar".to_string());
            }
            if let Some(grad_ref) = &tensor.grad {
                let grad_data = grad_ref.read().unwrap().clone();
                let grad_tensor = crate::heap::Tensor {
                    data: std::sync::Arc::new(std::sync::RwLock::new(grad_data)),
                    shape: tensor.shape.clone(),
                    strides: tensor.strides.clone(),
                    offset: tensor.offset,
                    requires_grad: false,
                    grad: None,
                    tape_node: None,
                };
                let grad_idx = heap.alloc(HeapData::Float64Array(grad_tensor));
                return Ok(Value::Float64Array(grad_idx));
            } else {
                return Ok(Value::Kosong);
            }
        }
        Err("Argumen harus berupa Float64Array".to_string())
    }

    fn matriks_backward_wrapper(ctx: &mut dyn VmContext, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != 1 {
            return Err("Matriks.backward membutuhkan 1 argumen".to_string());
        }
        if let Value::Float64Array(idx) = args[0] {
            let heap = ctx.get_heap_mut();
            
            // 1. Inisialisasi gradien tensor akhir dengan angka 1.0
            {
                let tensor = heap.get_f64_tensor_mut(idx);
                if !tensor.requires_grad {
                    return Err("Tensor ini tidak memiliki requires_grad = benar".to_string());
                }
                let data_len = tensor.data.read().unwrap().len();
                tensor.grad = Some(std::sync::Arc::new(std::sync::RwLock::new(vec![1.0; data_len])));
            }

            // 2. Klon tape (untuk menghindari mutability conflict)
            let nodes = heap.tape.nodes.clone();
            
            // 3. Eksekusi mundur melalui Tape
            for node in nodes.iter().rev() {
                // ... proses backward ...
                // Sederhanakan untuk contoh ini: Ambil grad dari node hasil
                let mut self_grad = None;
                if let Some(t) = heap.get_f64_tensor_opt(node.self_tensor_idx) {
                    if let Some(g) = &t.grad {
                        self_grad = Some(g.read().unwrap().clone());
                    }
                }
                
                if let Some(s_grad) = self_grad {
                    // Distribusikan ke parent berdasarkan op
                    match node.op {
                        crate::autograd::BackwardOp::Add => {
                            for parent_idx in &node.parents {
                                if let Some(p) = heap.get_f64_tensor_opt_mut(*parent_idx) {
                                    if p.grad.is_none() {
                                        p.grad = Some(std::sync::Arc::new(std::sync::RwLock::new(vec![0.0; s_grad.len()])));
                                    }
                                    let mut p_grad = p.grad.as_ref().unwrap().write().unwrap();
                                    for i in 0..p_grad.len() {
                                        p_grad[i] += s_grad[i];
                                    }
                                }
                            }
                        }
                        crate::autograd::BackwardOp::Mul => {
                            // C = A * B
                            // dC/dA = B -> dL/dA = dL/dC * B
                            if node.parents.len() == 2 {
                                let a_idx = node.parents[0];
                                let b_idx = node.parents[1];
                                
                                let (a_data, b_data) = {
                                    let p_a = heap.get_f64_tensor_opt(a_idx).unwrap();
                                    let p_b = heap.get_f64_tensor_opt(b_idx).unwrap();
                                    (p_a.data.read().unwrap().clone(), p_b.data.read().unwrap().clone())
                                };

                                if let Some(p_a) = heap.get_f64_tensor_opt_mut(a_idx) {
                                    if p_a.grad.is_none() {
                                        p_a.grad = Some(std::sync::Arc::new(std::sync::RwLock::new(vec![0.0; s_grad.len()])));
                                    }
                                    let mut p_grad = p_a.grad.as_ref().unwrap().write().unwrap();
                                    for i in 0..p_grad.len() {
                                        p_grad[i] += s_grad[i] * b_data[i]; // Sederhanakan tanpa broadcast
                                    }
                                }
                                
                                if let Some(p_b) = heap.get_f64_tensor_opt_mut(b_idx) {
                                    if p_b.grad.is_none() {
                                        p_b.grad = Some(std::sync::Arc::new(std::sync::RwLock::new(vec![0.0; s_grad.len()])));
                                    }
                                    let mut p_grad = p_b.grad.as_ref().unwrap().write().unwrap();
                                    for i in 0..p_grad.len() {
                                        p_grad[i] += s_grad[i] * a_data[i];
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            return Ok(Value::Kosong);
        }
        Err("Argumen harus berupa Float64Array".to_string())
    }

    let funcs = vec![
        ("dot", matriks_dot_wrapper as fn(&mut dyn VmContext, Vec<Value>) -> Result<Value, String>),
        ("transpose", matriks_transpose_wrapper),
        ("inverse", matriks_inverse_wrapper),
        ("determinant", matriks_determinant_wrapper),
        ("eigen", matriks_eigen_wrapper),
        ("grad", matriks_grad_wrapper),
        ("backward", matriks_backward_wrapper),
    ];

    for (name, func) in funcs {
        let func_obj = FungsiBawaanVM {
            nama: format!("Matriks.{}", name),
            func: std::sync::Arc::new(func),
        };
        let idx = vm.heap.alloc(HeapData::FungsiBawaan(func_obj));
        matriks_mod.insert(name.to_string(), Value::FungsiBawaan(idx));
    }

    let matriks_mod_idx = vm.heap.alloc(HeapData::Modul(matriks_mod));
    vm.set_global("Matriks".to_string(), Value::Modul(matriks_mod_idx));
}

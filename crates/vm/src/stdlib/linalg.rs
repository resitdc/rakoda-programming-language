use crate::heap::{HeapData, Tensor};
use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value, VmContext};
use nalgebra::DMatrix;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

fn tensor_to_dmatrix(t: &Tensor<f64>) -> Result<DMatrix<f64>, String> {
    if t.shape.is_empty() {
        return Err("Tensor kosong tidak dapat diubah ke matriks".to_string());
    } else if t.shape.len() == 1 {
        let rows = t.shape[0];
        let cols = 1;
        let data = t.data.read().unwrap();
        let mut flat = Vec::with_capacity(rows * cols);
        for i in 0..rows {
            let idx = t.offset + i * t.strides[0];
            flat.push(data[idx]);
        }
        Ok(DMatrix::from_row_iterator(rows, cols, flat.into_iter()))
    } else if t.shape.len() == 2 {
        let rows = t.shape[0];
        let cols = t.shape[1];
        let data = t.data.read().unwrap();
        let mut flat = Vec::with_capacity(rows * cols);
        for i in 0..rows {
            for j in 0..cols {
                let idx = t.offset + i * t.strides[0] + j * t.strides[1];
                flat.push(data[idx]);
            }
        }
        Ok(DMatrix::from_row_iterator(rows, cols, flat.into_iter()))
    } else {
        Err(format!("Hanya tensor 1D atau 2D yang dapat diubah ke matriks, tapi tensor ini memiliki bentuk {:?}", t.shape))
    }
}

fn dmatrix_to_tensor(m: &DMatrix<f64>) -> Tensor<f64> {
    let rows = m.nrows();
    let cols = m.ncols();
    let mut flat = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        for j in 0..cols {
            flat.push(m[(i, j)]);
        }
    }
    Tensor {
        data: Arc::new(RwLock::new(flat)),
        shape: vec![rows, cols],
        strides: vec![cols, 1],
        offset: 0,
    }
}

pub fn register(vm: &mut VM) {
    let mut module_dict = HashMap::new();

    macro_rules! register_func {
        ($name:expr, $func:expr) => {
            let func_obj = FungsiBawaanVM {
                nama: $name.to_string(),
                func: Arc::new($func),
            };
            let func_idx = vm.heap.alloc(HeapData::FungsiBawaan(func_obj));
            module_dict.insert($name.to_string(), Value::FungsiBawaan(func_idx));
        };
    }

    register_func!("invers", |ctx, args| {
        if args.len() != 1 { return Err("Linalg.invers membutuhkan 1 argumen matriks".to_string()); }
        let Value::Float64Array(idx) = args[0] else { return Err("Argumen harus berupa tensor/matriks angka".to_string()); };
        let tensor = ctx.get_heap_mut().get_f64_tensor(idx).clone();
        let mat = tensor_to_dmatrix(&tensor)?;
        
        let inv = mat.clone().try_inverse()
            .ok_or_else(|| "Matriks tidak memiliki invers (singular)".to_string())?;
            
        let res_tensor = dmatrix_to_tensor(&inv);
        let res_idx = ctx.get_heap_mut().alloc(HeapData::Float64Array(res_tensor));
        Ok(Value::Float64Array(res_idx))
    });

    register_func!("determinan", |ctx, args| {
        if args.len() != 1 { return Err("Linalg.determinan membutuhkan 1 argumen matriks".to_string()); }
        let Value::Float64Array(idx) = args[0] else { return Err("Argumen harus berupa tensor/matriks angka".to_string()); };
        let tensor = ctx.get_heap_mut().get_f64_tensor(idx).clone();
        let mat = tensor_to_dmatrix(&tensor)?;
        
        if mat.nrows() != mat.ncols() {
            return Err("Matriks harus bujursangkar untuk menghitung determinan".to_string());
        }
        let det = mat.determinant();
        Ok(Value::Angka(det))
    });

    register_func!("lu", |ctx, args| {
        if args.len() != 1 { return Err("Linalg.lu membutuhkan 1 argumen matriks".to_string()); }
        let Value::Float64Array(idx) = args[0] else { return Err("Argumen harus berupa tensor/matriks angka".to_string()); };
        let tensor = ctx.get_heap_mut().get_f64_tensor(idx).clone();
        let mat = tensor_to_dmatrix(&tensor)?;
        
        let lu = mat.lu();
        let l = lu.l();
        let u = lu.u();
        
        let mut heap = ctx.get_heap_mut();
        let l_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&l)));
        let u_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&u)));
        
        let arr_idx = heap.alloc(HeapData::Array(vec![
            Value::Float64Array(l_idx),
            Value::Float64Array(u_idx),
        ]));
        Ok(Value::Array(arr_idx))
    });

    register_func!("qr", |ctx, args| {
        if args.len() != 1 { return Err("Linalg.qr membutuhkan 1 argumen matriks".to_string()); }
        let Value::Float64Array(idx) = args[0] else { return Err("Argumen harus berupa tensor/matriks angka".to_string()); };
        let tensor = ctx.get_heap_mut().get_f64_tensor(idx).clone();
        let mat = tensor_to_dmatrix(&tensor)?;
        
        let qr = mat.qr();
        let q = qr.q();
        let r = qr.r();
        
        let mut heap = ctx.get_heap_mut();
        let q_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&q)));
        let r_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&r)));
        
        let arr_idx = heap.alloc(HeapData::Array(vec![
            Value::Float64Array(q_idx),
            Value::Float64Array(r_idx),
        ]));
        Ok(Value::Array(arr_idx))
    });

    register_func!("svd", |ctx, args| {
        if args.len() != 1 { return Err("Linalg.svd membutuhkan 1 argumen matriks".to_string()); }
        let Value::Float64Array(idx) = args[0] else { return Err("Argumen harus berupa tensor/matriks angka".to_string()); };
        let tensor = ctx.get_heap_mut().get_f64_tensor(idx).clone();
        let mat = tensor_to_dmatrix(&tensor)?;
        
        let svd = mat.svd(true, true);
        let u = svd.u.unwrap_or_else(|| DMatrix::zeros(0, 0));
        let mut s = DMatrix::zeros(svd.singular_values.nrows(), svd.singular_values.nrows());
        for i in 0..svd.singular_values.nrows() {
            s[(i, i)] = svd.singular_values[i];
        }
        let vt = svd.v_t.unwrap_or_else(|| DMatrix::zeros(0, 0));
        
        let mut heap = ctx.get_heap_mut();
        let u_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&u)));
        let s_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&s)));
        let vt_idx = heap.alloc(HeapData::Float64Array(dmatrix_to_tensor(&vt)));
        
        let arr_idx = heap.alloc(HeapData::Array(vec![
            Value::Float64Array(u_idx),
            Value::Float64Array(s_idx),
            Value::Float64Array(vt_idx),
        ]));
        Ok(Value::Array(arr_idx))
    });

    register_func!("cholesky", |ctx, args| {
        if args.len() != 1 { return Err("Linalg.cholesky membutuhkan 1 argumen matriks".to_string()); }
        let Value::Float64Array(idx) = args[0] else { return Err("Argumen harus berupa tensor/matriks angka".to_string()); };
        let tensor = ctx.get_heap_mut().get_f64_tensor(idx).clone();
        let mat = tensor_to_dmatrix(&tensor)?;
        
        let chol = mat.clone().cholesky()
            .ok_or_else(|| "Matriks bukan positif definit untuk dekomposisi Cholesky".to_string())?;
        
        let l = chol.l();
        let res_tensor = dmatrix_to_tensor(&l);
        let res_idx = ctx.get_heap_mut().alloc(HeapData::Float64Array(res_tensor));
        Ok(Value::Float64Array(res_idx))
    });

    register_func!("solve", |ctx, args| {
        if args.len() != 2 { return Err("Linalg.solve membutuhkan 2 argumen: matriks A dan matriks b".to_string()); }
        let Value::Float64Array(idx_a) = args[0] else { return Err("Argumen A harus berupa tensor/matriks angka".to_string()); };
        let Value::Float64Array(idx_b) = args[1] else { return Err("Argumen b harus berupa tensor/matriks angka".to_string()); };
        
        let tensor_a = ctx.get_heap_mut().get_f64_tensor(idx_a).clone();
        let tensor_b = ctx.get_heap_mut().get_f64_tensor(idx_b).clone();
        
        let mat_a = tensor_to_dmatrix(&tensor_a)?;
        let mat_b = tensor_to_dmatrix(&tensor_b)?;
        
        let decomp = mat_a.lu();
        let x = decomp.solve(&mat_b)
            .ok_or_else(|| "Gagal menyelesaikan sistem persamaan (matriks singular)".to_string())?;
            
        let res_tensor = dmatrix_to_tensor(&x);
        let res_idx = ctx.get_heap_mut().alloc(HeapData::Float64Array(res_tensor));
        Ok(Value::Float64Array(res_idx))
    });

    let dict_idx = vm.heap.alloc(HeapData::Kamus(module_dict));
    vm.set_global("Linalg".to_string(), Value::Kamus(dict_idx));
    vm.set_global("AljabarLinear".to_string(), Value::Kamus(dict_idx));
}

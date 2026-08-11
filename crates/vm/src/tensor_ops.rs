use crate::heap::{Heap, HeapData, Tensor};
use crate::opcodes::OpCode;
use crate::value::Value;
use std::sync::{Arc, RwLock};

pub fn apply_binary_op(heap: &mut Heap, a: Value, b: Value, op: OpCode) -> Result<Value, String> {
    match (a, b) {
        // --- SCALAR VS SCALAR ---
        (Value::Angka(a_val), Value::Angka(b_val)) => {
            let res = apply_scalar_op(a_val, b_val, op)?;
            Ok(Value::Angka(res))
        }

        // --- DUAL NUMBERS ---
        (Value::AngkaDual(ar, ag), Value::AngkaDual(br, bg)) => {
            let res = apply_dual_op(ar, ag, br, bg, op)?;
            Ok(Value::AngkaDual(res.0, res.1))
        }
        (Value::AngkaDual(ar, ag), Value::Angka(b_val)) => {
            let res = apply_dual_op(ar, ag, b_val, 0.0, op)?;
            Ok(Value::AngkaDual(res.0, res.1))
        }
        (Value::Angka(a_val), Value::AngkaDual(br, bg)) => {
            let res = apply_dual_op(a_val, 0.0, br, bg, op)?;
            Ok(Value::AngkaDual(res.0, res.1))
        }

        // --- KOMPLEKS VS SCALAR/KOMPLEKS ---
        (Value::Kompleks(ar, ai), Value::Angka(b_val)) => {
            let res = apply_complex_op(ar, ai, b_val, 0.0, op)?;
            Ok(Value::Kompleks(res.0, res.1))
        }
        (Value::Angka(a_val), Value::Kompleks(br, bi)) => {
            let res = apply_complex_op(a_val, 0.0, br, bi, op)?;
            Ok(Value::Kompleks(res.0, res.1))
        }
        (Value::Kompleks(ar, ai), Value::Kompleks(br, bi)) => {
            let res = apply_complex_op(ar, ai, br, bi, op)?;
            Ok(Value::Kompleks(res.0, res.1))
        }

        // --- STRING VS SCALAR/KOMPLEKS (Multiplication / Add) ---
        (Value::String(s_idx), Value::Angka(n)) | (Value::Angka(n), Value::String(s_idx)) => {
            if matches!(op, OpCode::Multiply) {
                let s = heap.get_string(s_idx).clone();
                let times = n as usize;
                let new_str = s.repeat(times);
                let new_idx = heap.alloc(HeapData::String(new_str));
                Ok(Value::String(new_idx))
            } else if matches!(op, OpCode::Add) {
                let s1 = a.to_string(heap);
                let s2 = b.to_string(heap);
                let new_idx = heap.alloc(HeapData::String(format!("{}{}", s1, s2)));
                Ok(Value::String(new_idx))
            } else {
                Err("Operasi tidak didukung antara teks dan angka".to_string())
            }
        }
        (Value::String(_), Value::Kompleks(_, _)) | (Value::Kompleks(_, _), Value::String(_)) => {
            if matches!(op, OpCode::Add) {
                let s1 = a.to_string(heap);
                let s2 = b.to_string(heap);
                let new_idx = heap.alloc(HeapData::String(format!("{}{}", s1, s2)));
                Ok(Value::String(new_idx))
            } else {
                Err("Operasi tidak didukung antara teks dan bilangan kompleks".to_string())
            }
        }
        
        // --- STRING VS STRING (Addition) ---
        (Value::String(s1_idx), Value::String(s2_idx)) => {
            if matches!(op, OpCode::Add) {
                let s1 = heap.get_string(s1_idx).clone();
                let s2 = heap.get_string(s2_idx).clone();
                let new_idx = heap.alloc(HeapData::String(format!("{}{}", s1, s2)));
                Ok(Value::String(new_idx))
            } else {
                Err("Hanya operasi penambahan (+) yang didukung untuk dua teks".to_string())
            }
        }

        // --- ARRAY REGULER VS SCALAR ---
        (Value::Array(arr_idx), Value::Angka(n)) => {
            let elements = heap.get_array(arr_idx).clone();
            let mut new_elements = Vec::with_capacity(elements.len());
            for el in elements {
                new_elements.push(apply_binary_op(heap, el.clone(), Value::Angka(n), op.clone())?);
            }
            let new_idx = heap.alloc(HeapData::Array(new_elements));
            Ok(Value::Array(new_idx))
        }
        (Value::Angka(n), Value::Array(arr_idx)) => {
            let elements = heap.get_array(arr_idx).clone();
            let mut new_elements = Vec::with_capacity(elements.len());
            for el in elements {
                new_elements.push(apply_binary_op(heap, Value::Angka(n), el.clone(), op.clone())?);
            }
            let new_idx = heap.alloc(HeapData::Array(new_elements));
            Ok(Value::Array(new_idx))
        }

        // --- ARRAY REGULER VS ARRAY REGULER ---
        (Value::Array(arr1_idx), Value::Array(arr2_idx)) => {
            let el1 = heap.get_array(arr1_idx).clone();
            let el2 = heap.get_array(arr2_idx).clone();
            if el1.len() != el2.len() {
                return Err(format!("Ukuran array tidak sama: {} dan {}", el1.len(), el2.len()));
            }
            let mut new_elements = Vec::with_capacity(el1.len());
            for i in 0..el1.len() {
                new_elements.push(apply_binary_op(heap, el1[i].clone(), el2[i].clone(), op.clone())?);
            }
            let new_idx = heap.alloc(HeapData::Array(new_elements));
            Ok(Value::Array(new_idx))
        }

        // --- TENSORS ---
        (Value::Float64Array(idx1), Value::Angka(n)) => {
            let t = heap.get_f64_tensor(idx1);
            let result_tensor = broadcast_tensor_scalar(t, n, op)?;
            let new_idx = heap.alloc(HeapData::Float64Array(result_tensor));
            Ok(Value::Float64Array(new_idx))
        }
        (Value::Angka(n), Value::Float64Array(idx2)) => {
            let t = heap.get_f64_tensor(idx2);
            let result_tensor = broadcast_scalar_tensor(n, t, op)?;
            let new_idx = heap.alloc(HeapData::Float64Array(result_tensor));
            Ok(Value::Float64Array(new_idx))
        }
        (Value::Float64Array(idx1), Value::Float64Array(idx2)) => {
            let t1 = heap.get_f64_tensor(idx1);
            let t2 = heap.get_f64_tensor(idx2);
            let result_tensor = broadcast_tensor_tensor(t1, t2, op)?;
            let new_idx = heap.alloc(HeapData::Float64Array(result_tensor));
            Ok(Value::Float64Array(new_idx))
        }

        _ => Err("Operasi matematis tidak didukung untuk tipe data ini".to_string()),
    }
}

fn apply_complex_op(ar: f64, ai: f64, br: f64, bi: f64, op: OpCode) -> Result<(f64, f64), String> {
    match op {
        OpCode::Add => Ok((ar + br, ai + bi)),
        OpCode::Subtract => Ok((ar - br, ai - bi)),
        OpCode::Multiply => Ok((ar * br - ai * bi, ar * bi + ai * br)),
        OpCode::Divide => {
            let den = br * br + bi * bi;
            if den == 0.0 {
                Err("Pembagian dengan nol pada bilangan kompleks".to_string())
            } else {
                Ok(((ar * br + ai * bi) / den, (ai * br - ar * bi) / den))
            }
        }
        _ => Err("Operasi matematis tidak didukung untuk bilangan kompleks".to_string()),
    }
}

fn apply_dual_op(ar: f64, ag: f64, br: f64, bg: f64, op: OpCode) -> Result<(f64, f64), String> {
    match op {
        OpCode::Add => Ok((ar + br, ag + bg)),
        OpCode::Subtract => Ok((ar - br, ag - bg)),
        OpCode::Multiply => Ok((ar * br, ag * br + ar * bg)),
        OpCode::Divide => {
            if br == 0.0 {
                Err("Pembagian dengan nol".to_string())
            } else {
                Ok((ar / br, (ag * br - ar * bg) / (br * br)))
            }
        }
        OpCode::Power => {
            let r = ar.powf(br);
            let d = if bg == 0.0 {
                br * ar.powf(br - 1.0) * ag
            } else {
                r * (bg * ar.ln() + br * ag / ar)
            };
            Ok((r, d))
        }
        _ => Err("Operasi matematis tidak didukung untuk bilangan dual (AutoGrad)".to_string()),
    }
}

fn apply_scalar_op(a: f64, b: f64, op: OpCode) -> Result<f64, String> {
    match op {
        OpCode::Add => Ok(a + b),
        OpCode::Subtract => Ok(a - b),
        OpCode::Multiply => Ok(a * b),
        OpCode::Divide => {
            if b == 0.0 {
                Err("Pembagian dengan nol".to_string())
            } else {
                Ok(a / b)
            }
        }
        OpCode::Modulus => Ok(a % b),
        OpCode::Power => Ok(a.powf(b)),
        OpCode::BitwiseOr => Ok((a as i32 | b as i32) as f64),
        _ => Err("Operator skalar tidak valid".to_string()),
    }
}

fn broadcast_tensor_scalar(t: &Tensor<f64>, scalar: f64, op: OpCode) -> Result<Tensor<f64>, String> {
    let data = t.data.read().unwrap();
    let mut new_data = Vec::with_capacity(data.len());

    // Iterasi dari data linear.
    // Jika tensor merupakan slice (strides != standard), idealnya kita menggunakan iterator khusus.
    // Untuk saat ini, asumsikan data contiguous.
    for &val in data.iter() {
        new_data.push(apply_scalar_op(val, scalar, op.clone())?);
    }

    Ok(Tensor {
        data: Arc::new(RwLock::new(new_data)),
        shape: t.shape.clone(),
        strides: t.strides.clone(),
        offset: 0,
    })
}

fn broadcast_scalar_tensor(scalar: f64, t: &Tensor<f64>, op: OpCode) -> Result<Tensor<f64>, String> {
    let data = t.data.read().unwrap();
    let mut new_data = Vec::with_capacity(data.len());

    for &val in data.iter() {
        new_data.push(apply_scalar_op(scalar, val, op.clone())?);
    }

    Ok(Tensor {
        data: Arc::new(RwLock::new(new_data)),
        shape: t.shape.clone(),
        strides: t.strides.clone(),
        offset: 0,
    })
}

fn broadcast_tensor_tensor(t1: &Tensor<f64>, t2: &Tensor<f64>, op: OpCode) -> Result<Tensor<f64>, String> {
    // Algoritma Broadcasting Otomatis (NumPy Style)
    // 1. Pad dimensi yang lebih kecil dengan 1 di depan
    let mut s1 = t1.shape.clone();
    let mut s2 = t2.shape.clone();

    while s1.len() < s2.len() {
        s1.insert(0, 1);
    }
    while s2.len() < s1.len() {
        s2.insert(0, 1);
    }

    // 2. Tentukan shape output
    let mut out_shape = Vec::with_capacity(s1.len());
    for i in 0..s1.len() {
        if s1[i] == s2[i] {
            out_shape.push(s1[i]);
        } else if s1[i] == 1 {
            out_shape.push(s2[i]);
        } else if s2[i] == 1 {
            out_shape.push(s1[i]);
        } else {
            return Err(format!(
                "Dimensi tidak bisa di-broadcast bersama: {:?} dan {:?}",
                t1.shape, t2.shape
            ));
        }
    }

    let out_size: usize = out_shape.iter().product();
    let mut new_data = vec![0.0; out_size];

    let d1 = t1.data.read().unwrap();
    let d2 = t2.data.read().unwrap();

    // Fungsi utilitas untuk memetakan flat_index output ke flat_index input
    // Ini mendukung broadcasting dan slicing sekaligus
    let get_input_idx = |flat_idx: usize, in_shape: &[usize], strides: &[usize], offset: usize| -> usize {
        let mut idx = offset;
        let mut rem = flat_idx;
        // Hitung koordinat N-dimensi
        for i in (0..out_shape.len()).rev() {
            let size = out_shape[i];
            let coord = rem % size;
            rem /= size;

            // Jika dimensi asli adalah 1 (broadcasted), koordinatnya selalu 0
            let in_coord = if in_shape.len() > i && in_shape[in_shape.len() - out_shape.len() + i] == 1 {
                0
            } else {
                coord
            };
            
            if in_shape.len() > i {
                 let s_idx = strides.len() - out_shape.len() + i;
                 idx += in_coord * strides[s_idx];
            }
        }
        idx
    };

    // Pad in_shape dan in_strides dengan 1 jika perlu
    let mut in_shape1 = t1.shape.clone();
    let mut in_strides1 = t1.strides.clone();
    while in_shape1.len() < out_shape.len() {
        in_shape1.insert(0, 1);
        in_strides1.insert(0, 0);
    }
    
    let mut in_shape2 = t2.shape.clone();
    let mut in_strides2 = t2.strides.clone();
    while in_shape2.len() < out_shape.len() {
        in_shape2.insert(0, 1);
        in_strides2.insert(0, 0);
    }

    for i in 0..out_size {
        let idx1 = get_input_idx(i, &in_shape1, &in_strides1, t1.offset);
        let idx2 = get_input_idx(i, &in_shape2, &in_strides2, t2.offset);

        let v1 = d1[idx1];
        let v2 = d2[idx2];
        new_data[i] = apply_scalar_op(v1, v2, op.clone())?;
    }

    // Strides untuk output (selalu contiguous)
    let mut out_strides = vec![1; out_shape.len()];
    let mut current_stride = 1;
    for i in (0..out_shape.len()).rev() {
        out_strides[i] = current_stride;
        current_stride *= out_shape[i];
    }

    Ok(Tensor {
        data: Arc::new(RwLock::new(new_data)),
        shape: out_shape,
        strides: out_strides,
        offset: 0,
    })
}

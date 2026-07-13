//! VM adapter: conversion helpers between VM Value/Heap and stdlib's NilaiRpl.
//! Thin layer — no business logic. Each VM stdlib module converts args/results
//! and calls the pure functions from the `stdlib` crate directly.

use crate::heap::{Heap, HeapData};
use crate::value::Value;
use std::collections::HashMap;
use stdlib::jenis::NilaiRpl;

/// Convert NilaiRpl → VM Value (allocates into heap).
pub fn nilai_ke_value(nilai: &NilaiRpl, heap: &mut Heap) -> Value {
    match nilai {
        NilaiRpl::Angka(n) => Value::Angka(*n),
        NilaiRpl::Boolean(b) => Value::Boolean(*b),
        NilaiRpl::Kosong => Value::Kosong,
        NilaiRpl::Teks(s) => {
            let idx = heap.alloc(HeapData::String(s.clone()));
            Value::String(idx)
        }
        NilaiRpl::Daftar(items) => {
            let values: Vec<Value> = items.iter().map(|v| nilai_ke_value(v, heap)).collect();
            let idx = heap.alloc(HeapData::Array(values));
            Value::Array(idx)
        }
        NilaiRpl::Kamus(entries) => {
            let mut map = HashMap::new();
            for (k, v) in entries {
                map.insert(k.clone(), nilai_ke_value(v, heap));
            }
            let idx = heap.alloc(HeapData::Kamus(map));
            Value::Kamus(idx)
        }
    }
}

/// Convert VM Value → NilaiRpl (reads from heap).
pub fn value_ke_nilai(val: &Value, heap: &Heap) -> NilaiRpl {
    match val {
        Value::Angka(n) => NilaiRpl::Angka(*n),
        Value::Boolean(b) => NilaiRpl::Boolean(*b),
        Value::Kosong => NilaiRpl::Kosong,
        Value::String(idx) => NilaiRpl::Teks(heap.get_string(*idx).clone()),
        Value::Array(idx) => {
            let items: Vec<NilaiRpl> = heap
                .get_array(*idx)
                .iter()
                .map(|v| value_ke_nilai(v, heap))
                .collect();
            NilaiRpl::Daftar(items)
        }
        Value::Kamus(idx) => {
            let mut map = HashMap::new();
            for (k, v) in heap.get_kamus(*idx) {
                map.insert(k.clone(), value_ke_nilai(v, heap));
            }
            NilaiRpl::Kamus(map)
        }
        _ => NilaiRpl::Kosong,
    }
}

use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use ocrs::{OcrEngine, OcrEngineParams, ImageSource, DimOrder};
use rten_imageio::read_image;
use rten::Model;
use rten_tensor::prelude::*;

const DETECTION_MODEL: &[u8] = include_bytes!("models/text-detection.rten");
const RECOGNITION_MODEL: &[u8] = include_bytes!("models/text-recognition.rten");

fn inisialisasi_engine() -> Result<OcrEngine, String> {
    OcrEngine::new(OcrEngineParams {
        detection_model: Some(Model::load_static_slice(DETECTION_MODEL).map_err(|e| e.to_string())?),
        recognition_model: Some(Model::load_static_slice(RECOGNITION_MODEL).map_err(|e| e.to_string())?),
        ..Default::default()
    })
    .map_err(|e| format!("Gagal inisialisasi mesin OCR: {}", e))
}

pub fn register(vm: &mut VM) {
    let mut map = HashMap::new();

    let baca_func = FungsiBawaanVM {
        nama: "ocr.baca".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err("ocr.baca(path_gambar) membutuhkan 1 argumen".to_string());
            }

            let path_str = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen path_gambar harus berupa string".to_string()),
            };

            if !Path::new(&path_str).exists() {
                return Err(format!("File gambar tidak ditemukan: {}", path_str));
            }

            let engine = match inisialisasi_engine() {
                Ok(e) => e,
                Err(e) => return Err(e),
            };

            let img = match read_image(&path_str) {
                Ok(i) => i,
                Err(e) => return Err(format!("Gagal membaca gambar: {}", e)),
            };

            let img_source = match ImageSource::from_tensor(img.view(), DimOrder::Chw) {
                Ok(src) => src,
                Err(e) => return Err(format!("Gagal membuat sumber gambar: {:?}", e)),
            };

            let ocr_input = match engine.prepare_input(img_source) {
                Ok(i) => i,
                Err(e) => return Err(format!("Gagal memproses gambar untuk OCR: {}", e)),
            };

            let word_rects = match engine.detect_words(&ocr_input) {
                Ok(r) => r,
                Err(e) => return Err(format!("Gagal mendeteksi teks: {}", e)),
            };

            let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

            let texts = match engine.recognize_text(&ocr_input, &line_rects) {
                Ok(t) => t,
                Err(e) => return Err(format!("Gagal mengenali teks: {}", e)),
            };

            let mut result_string = String::new();
            for (line_idx, line) in texts.iter().flatten().enumerate() {
                if line_idx > 0 {
                    result_string.push('\n');
                }
                result_string.push_str(&line.to_string());
            }

            let ptr = ctx.get_heap_mut().alloc(crate::heap::HeapData::String(result_string));
            Ok(Value::String(ptr))
        }),
    };

    let baca_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(baca_func));
    map.insert("baca".to_string(), Value::FungsiBawaan(baca_idx));

    let ptr = vm.heap.alloc(crate::heap::HeapData::Kamus(map));
    vm.environments[0].insert("gambar".to_string(), Value::Kamus(ptr));
}

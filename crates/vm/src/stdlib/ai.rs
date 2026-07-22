use crate::value::{FungsiBawaanVM, Value};
use crate::machine::VM;
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(vm: &mut VM) {
    let mut map = HashMap::new();

    // Inisialisasi properti kosong
    map.insert(
        "_penyedia".to_string(),
        Value::String(vm.heap.alloc(crate::heap::HeapData::String("".to_string()))),
    );
    map.insert(
        "_kunci".to_string(),
        Value::String(vm.heap.alloc(crate::heap::HeapData::String("".to_string()))),
    );

    // ai.penyedia("openai")
    let penyedia_func = FungsiBawaanVM {
        nama: "ai.penyedia".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'penyedia' membutuhkan 1 argumen teks (nama penyedia).".to_string());
            }
            
            let provider = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen penyedia harus berupa teks.".to_string()),
            };

            let provider_lower = provider.to_lowercase();
            if !["gemini", "openai", "anthropic", "glm", "deepseek"].contains(&provider_lower.as_str()) {
                return Err(format!("Provider '{}' tidak didukung. Dukungan: gemini, openai, anthropic, glm, deepseek.", provider));
            }

            // Kita menyimpan provider global ini di environment global "ai" 
            if let Some(vm) = ctx.as_any().downcast_mut::<VM>() {
                // Cari global variable 'ai'
                if let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned() {
                    let provider_idx = vm.heap.alloc(crate::heap::HeapData::String(provider_lower));
                    let k = vm.heap.get_kamus_mut(ai_idx);
                    k.insert("_penyedia".to_string(), Value::String(provider_idx));
                }
            }
            
            Ok(Value::Kosong)
        }),
    };
    let penyedia_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(penyedia_func));
    map.insert("penyedia".to_string(), Value::FungsiBawaan(penyedia_idx));

    // ai.kunci("...") / ai.key("...")
    let kunci_func = FungsiBawaanVM {
        nama: "ai.kunci".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'kunci' membutuhkan 1 argumen teks (api key).".to_string());
            }
            
            let key = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen kunci harus berupa teks.".to_string()),
            };

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>() {
                if let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned() {
                    let key_idx = vm.heap.alloc(crate::heap::HeapData::String(key));
                    let k = vm.heap.get_kamus_mut(ai_idx);
                    k.insert("_kunci".to_string(), Value::String(key_idx));
                }
            }
            
            Ok(Value::Kosong)
        }),
    };
    let kunci_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(kunci_func));
    
    map.insert("kunci".to_string(), Value::FungsiBawaan(kunci_idx));
    map.insert("key".to_string(), Value::FungsiBawaan(kunci_idx));

    // ai.tanya("...")
    let tanya_func = FungsiBawaanVM {
        nama: "ai.tanya".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'tanya' membutuhkan 1 argumen teks (prompt).".to_string());
            }
            
            let prompt = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen tanya harus berupa teks.".to_string()),
            };

            let mut provider = String::new();
            let mut key = String::new();

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>() {
                if let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned() {
                    let k = vm.heap.get_kamus(ai_idx);
                    if let Some(Value::String(p_idx)) = k.get("_penyedia") {
                        provider = vm.heap.get_string(*p_idx).clone();
                    }
                    if let Some(Value::String(k_idx)) = k.get("_kunci") {
                        key = vm.heap.get_string(*k_idx).clone();
                    }
                }
            }

            if provider.is_empty() {
                return Err("Provider AI belum diatur. Gunakan ai.penyedia('nama_penyedia').".to_string());
            }
            if key.is_empty() {
                return Err("Kunci API belum diatur. Gunakan ai.kunci('kunci_rahasia').".to_string());
            }

            let response_text = call_ai_api(&provider, &key, &prompt).map_err(|e| format!("Gagal menghubungi API AI: {}", e))?;
            
            let res_idx = ctx.get_heap_mut().alloc(crate::heap::HeapData::String(response_text));
            Ok(Value::String(res_idx))
        }),
    };
    let tanya_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(tanya_func));
    map.insert("tanya".to_string(), Value::FungsiBawaan(tanya_idx));

    let kamus_idx = vm.heap.alloc(crate::heap::HeapData::Kamus(map));
    vm.set_global("ai".to_string(), Value::Kamus(kamus_idx));
}

fn call_ai_api(provider: &str, api_key: &str, prompt: &str) -> Result<String, String> {
    match provider {
        "gemini" => call_gemini(api_key, prompt),
        "openai" => call_openai(api_key, prompt, "api.openai.com", "gpt-4o"),
        "anthropic" => call_anthropic(api_key, prompt),
        "glm" => call_openai(api_key, prompt, "open.bigmodel.cn/api/paas/v4", "glm-4"),
        "deepseek" => call_openai(api_key, prompt, "api.deepseek.com", "deepseek-chat"),
        _ => Err(format!("Provider '{}' tidak didukung.", provider)),
    }
}

// -----------------------------------------------------------------------------
// Provider implementations
// -----------------------------------------------------------------------------

fn call_gemini(api_key: &str, prompt: &str) -> Result<String, String> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);
    
    let json_body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let resp = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(json_body.to_string())
        .map_err(|e| e.to_string())?;

    let body_str = resp.into_body().read_to_string().map_err(|e| e.to_string())?;
    let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;
    
    if let Some(text) = body["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Format respons Gemini tidak sesuai.".to_string())
    }
}

fn call_openai(api_key: &str, prompt: &str, base_url: &str, default_model: &str) -> Result<String, String> {
    let url = format!("https://{}/v1/chat/completions", base_url);
    
    let json_body = serde_json::json!({
        "model": default_model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let auth_header = format!("Bearer {}", api_key);
    let resp = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &auth_header)
        .send(json_body.to_string())
        .map_err(|e| e.to_string())?;

    let body_str = resp.into_body().read_to_string().map_err(|e| e.to_string())?;
    let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;
    
    if let Some(text) = body["choices"][0]["message"]["content"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Format respons OpenAI-compatible tidak sesuai.".to_string())
    }
}

fn call_anthropic(api_key: &str, prompt: &str) -> Result<String, String> {
    let url = "https://api.anthropic.com/v1/messages";
    
    let json_body = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 1024,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let resp = ureq::post(url)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send(json_body.to_string())
        .map_err(|e| e.to_string())?;

    let body_str = resp.into_body().read_to_string().map_err(|e| e.to_string())?;
    let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;
    
    if let Some(text) = body["content"][0]["text"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Format respons Anthropic tidak sesuai.".to_string())
    }
}

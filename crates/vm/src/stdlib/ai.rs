use crate::machine::VM;
use crate::value::{FungsiBawaanVM, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(vm: &mut VM) {
    let mut map = HashMap::new();

    map.insert(
        "_penyedia".to_string(),
        Value::String(vm.heap.alloc(crate::heap::HeapData::String("".to_string()))),
    );
    map.insert(
        "_kunci".to_string(),
        Value::String(vm.heap.alloc(crate::heap::HeapData::String("".to_string()))),
    );
    map.insert(
        "_url".to_string(),
        Value::String(vm.heap.alloc(crate::heap::HeapData::String("".to_string()))),
    );
    map.insert(
        "_model".to_string(),
        Value::String(vm.heap.alloc(crate::heap::HeapData::String("".to_string()))),
    );

    // ai.penyedia("openai")
    let penyedia_func = FungsiBawaanVM {
        nama: "ai.penyedia".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err(
                    "Fungsi 'penyedia' membutuhkan 1 argumen teks (nama penyedia).".to_string(),
                );
            }

            let provider = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen penyedia harus berupa teks.".to_string()),
            };

            let provider_lower = provider.to_lowercase();

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>() {
                if let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned() {
                    let provider_idx = vm.heap.alloc(crate::heap::HeapData::String(provider_lower));
                    let k = vm.heap.get_kamus_mut(ai_idx);
                    k.insert("_penyedia".to_string(), Value::String(provider_idx));
                }
            }

            Ok(Value::Kosong)
        }),
    };
    let penyedia_idx = vm
        .heap
        .alloc(crate::heap::HeapData::FungsiBawaan(penyedia_func));
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

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>()
                && let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned()
            {
                let key_idx = vm.heap.alloc(crate::heap::HeapData::String(key));
                let k = vm.heap.get_kamus_mut(ai_idx);
                k.insert("_kunci".to_string(), Value::String(key_idx));
            }

            Ok(Value::Kosong)
        }),
    };
    let kunci_idx = vm
        .heap
        .alloc(crate::heap::HeapData::FungsiBawaan(kunci_func));

    map.insert("kunci".to_string(), Value::FungsiBawaan(kunci_idx));
    map.insert("key".to_string(), Value::FungsiBawaan(kunci_idx));

    // ai.url("...")
    let url_func = FungsiBawaanVM {
        nama: "ai.url".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'url' membutuhkan 1 argumen teks (URL endpoint).".to_string());
            }

            let url_str = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen url harus berupa teks.".to_string()),
            };

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>()
                && let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned()
            {
                let u_idx = vm.heap.alloc(crate::heap::HeapData::String(url_str));
                let k = vm.heap.get_kamus_mut(ai_idx);
                k.insert("_url".to_string(), Value::String(u_idx));
            }

            Ok(Value::Kosong)
        }),
    };
    let url_idx = vm.heap.alloc(crate::heap::HeapData::FungsiBawaan(url_func));
    map.insert("url".to_string(), Value::FungsiBawaan(url_idx));

    // ai.model("...")
    let model_func = FungsiBawaanVM {
        nama: "ai.model".to_string(),
        func: Arc::new(|ctx, args| {
            if args.len() != 1 {
                return Err("Fungsi 'model' membutuhkan 1 argumen teks (nama model).".to_string());
            }

            let model_str = match &args[0] {
                Value::String(idx) => ctx.get_heap_mut().get_string(*idx).clone(),
                _ => return Err("Argumen model harus berupa teks.".to_string()),
            };

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>()
                && let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned()
            {
                let m_idx = vm.heap.alloc(crate::heap::HeapData::String(model_str));
                let k = vm.heap.get_kamus_mut(ai_idx);
                k.insert("_model".to_string(), Value::String(m_idx));
            }

            Ok(Value::Kosong)
        }),
    };
    let model_idx = vm
        .heap
        .alloc(crate::heap::HeapData::FungsiBawaan(model_func));
    map.insert("model".to_string(), Value::FungsiBawaan(model_idx));

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
            let mut custom_url = String::new();
            let mut custom_model = String::new();

            if let Some(vm) = ctx.as_any().downcast_mut::<VM>()
                && let Some(Value::Kamus(ai_idx)) = vm.environments[0].get("ai").cloned()
            {
                let k = vm.heap.get_kamus(ai_idx);
                if let Some(Value::String(p_idx)) = k.get("_penyedia") {
                    provider = vm.heap.get_string(*p_idx).clone();
                }
                if let Some(Value::String(k_idx)) = k.get("_kunci") {
                    key = vm.heap.get_string(*k_idx).clone();
                }
                if let Some(Value::String(u_idx)) = k.get("_url") {
                    custom_url = vm.heap.get_string(*u_idx).clone();
                }
                if let Some(Value::String(m_idx)) = k.get("_model") {
                    custom_model = vm.heap.get_string(*m_idx).clone();
                }
            }

            if provider.is_empty() {
                return Err(
                    "Provider AI belum diatur. Gunakan ai.penyedia('nama_penyedia').".to_string(),
                );
            }
            if key.is_empty() && !provider.eq_ignore_ascii_case("ollama") && custom_url.is_empty() {
                return Err(
                    "Kunci API belum diatur. Gunakan ai.kunci('kunci_rahasia').".to_string()
                );
            }

            let response_text = call_ai_api(&provider, &key, &prompt, &custom_url, &custom_model)
                .map_err(|e| format!("Gagal menghubungi API AI: {}", e))?;

            let res_idx = ctx
                .get_heap_mut()
                .alloc(crate::heap::HeapData::String(response_text));
            Ok(Value::String(res_idx))
        }),
    };
    let tanya_idx = vm
        .heap
        .alloc(crate::heap::HeapData::FungsiBawaan(tanya_func));
    map.insert("tanya".to_string(), Value::FungsiBawaan(tanya_idx));

    let kamus_idx = vm.heap.alloc(crate::heap::HeapData::Kamus(map));
    vm.set_global("ai".to_string(), Value::Kamus(kamus_idx));
}

fn call_ai_api(
    provider: &str,
    api_key: &str,
    prompt: &str,
    custom_url: &str,
    custom_model: &str,
) -> Result<String, String> {
    match provider {
        "gemini" => call_gemini(api_key, prompt, custom_model),
        "openai" => call_openai(
            api_key,
            prompt,
            if custom_url.is_empty() {
                "https://api.openai.com/v1/chat/completions"
            } else {
                custom_url
            },
            if custom_model.is_empty() {
                "gpt-4o"
            } else {
                custom_model
            },
        ),
        "anthropic" => call_anthropic(api_key, prompt, custom_model),
        "glm" => call_openai(
            api_key,
            prompt,
            if custom_url.is_empty() {
                "https://open.bigmodel.cn/api/paas/v4/chat/completions"
            } else {
                custom_url
            },
            if custom_model.is_empty() {
                "glm-4"
            } else {
                custom_model
            },
        ),
        "deepseek" => call_openai(
            api_key,
            prompt,
            if custom_url.is_empty() {
                "https://api.deepseek.com/v1/chat/completions"
            } else {
                custom_url
            },
            if custom_model.is_empty() {
                "deepseek-chat"
            } else {
                custom_model
            },
        ),
        "ollama" => call_openai(
            api_key,
            prompt,
            if custom_url.is_empty() {
                "http://localhost:11434/v1/chat/completions"
            } else {
                custom_url
            },
            if custom_model.is_empty() {
                "llama3"
            } else {
                custom_model
            },
        ),
        // Fallback untuk provider custom lainnya yang kompatibel dengan format OpenAI
        _ => call_openai(
            api_key,
            prompt,
            if custom_url.is_empty() {
                "https://api.openai.com/v1/chat/completions"
            } else {
                custom_url
            },
            if custom_model.is_empty() {
                "gpt-3.5-turbo"
            } else {
                custom_model
            },
        ),
    }
}

// -----------------------------------------------------------------------------
// Provider implementations
// -----------------------------------------------------------------------------

fn call_gemini(api_key: &str, prompt: &str, custom_model: &str) -> Result<String, String> {
    let model = if custom_model.is_empty() {
        "gemini-1.5-flash"
    } else {
        custom_model
    };
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let json_body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let resp = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(json_body.to_string())
        .map_err(|e| e.to_string())?;

    let body_str = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;

    if let Some(text) = body["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Format respons Gemini tidak sesuai.".to_string())
    }
}

fn call_openai(api_key: &str, prompt: &str, url: &str, model: &str) -> Result<String, String> {
    let json_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let auth_header = format!("Bearer {}", api_key);
    let resp = ureq::post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", &auth_header)
        .send(json_body.to_string())
        .map_err(|e| e.to_string())?;

    let body_str = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;

    if let Some(text) = body["choices"][0]["message"]["content"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Format respons OpenAI-compatible tidak sesuai.".to_string())
    }
}

fn call_anthropic(api_key: &str, prompt: &str, custom_model: &str) -> Result<String, String> {
    let url = "https://api.anthropic.com/v1/messages";
    let model = if custom_model.is_empty() {
        "claude-3-haiku-20240307"
    } else {
        custom_model
    };

    let json_body = serde_json::json!({
        "model": model,
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

    let body_str = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let body: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;

    if let Some(text) = body["content"][0]["text"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Format respons Anthropic tidak sesuai.".to_string())
    }
}

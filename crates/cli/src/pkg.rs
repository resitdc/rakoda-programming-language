use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct PaketConfig {
    pub nama: String,
    pub versi: String,
    pub penulis: String,
    pub titik_masuk: String,
    #[serde(default)]
    pub dependensi: HashMap<String, String>,
}

impl Default for PaketConfig {
    fn default() -> Self {
        Self {
            nama: "proyek_baru".to_string(),
            versi: "1.0.0".to_string(),
            penulis: "Siswa SMK".to_string(),
            titik_masuk: "main.rpl".to_string(),
            dependensi: HashMap::new(),
        }
    }
}

pub fn inisialisasi(cwd: &Path) -> Result<()> {
    let rpl_json_path = cwd.join("rpl.json");
    if rpl_json_path.exists() {
        println!("⚠️  File rpl.json sudah ada.");
        return Ok(());
    }

    let default_config = PaketConfig {
        nama: cwd
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        ..Default::default()
    };

    let json_string = serde_json::to_string_pretty(&default_config)?;
    std::fs::write(&rpl_json_path, json_string).context("Gagal membuat rpl.json")?;

    println!("✅ Berhasil membuat rpl.json");

    let main_rpl_path = cwd.join("main.rpl");
    if !main_rpl_path.exists() {
        let default_main = "cetak \"Halo Dunia dari RPL!\"\n";
        std::fs::write(&main_rpl_path, default_main).context("Gagal membuat main.rpl")?;
        println!("✅ Berhasil membuat main.rpl");
    }

    Ok(())
}

pub async fn instal(paket: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let rpl_json_path = cwd.join("rpl.json");

    let mut config = if rpl_json_path.exists() {
        let content = tokio::fs::read_to_string(&rpl_json_path).await?;
        serde_json::from_str::<PaketConfig>(&content)?
    } else {
        PaketConfig {
            nama: cwd
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        }
    };

    if let Some(p) = paket {
        let parts: Vec<&str> = p.split(':').collect();
        let nama = if parts.len() > 1 {
            parts[1].split('/').next_back().unwrap_or(&p).to_string()
        } else {
            p.clone()
        };
        config.dependensi.insert(nama, p);
        let json_string = serde_json::to_string_pretty(&config)?;
        tokio::fs::write(&rpl_json_path, json_string).await?;
    }

    let mut installed = HashSet::new();
    let rpl_modules_dir = cwd.join("rpl_modules");
    if !rpl_modules_dir.exists() {
        tokio::fs::create_dir_all(&rpl_modules_dir).await?;
    }

    let dependensi = config.dependensi.clone();
    for (nama, url) in dependensi.iter() {
        download_and_extract_recursive(
            nama.clone(),
            url.clone(),
            rpl_modules_dir.clone(),
            &mut installed,
        )
        .await?;
    }

    println!("✅ Selesai menginstal dependensi!");
    Ok(())
}

use std::fs as std_fs;
use std::future::Future;
use std::pin::Pin;

fn download_and_extract_recursive<'a>(
    nama: String,
    url_ref: String,
    modules_dir: std::path::PathBuf,
    installed: &'a mut HashSet<String>,
) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        if installed.contains(&nama) {
            return Ok(());
        }

        println!("⬇️  Mengunduh paket '{}'...", nama);

        let download_url = if url_ref.starts_with("github:") {
            let repo = url_ref.trim_start_matches("github:");
            format!("https://github.com/{}/archive/refs/heads/main.zip", repo)
        } else {
            url_ref.to_string()
        };

        let response = reqwest::get(&download_url).await?;
        if !response.status().is_success() {
            anyhow::bail!("Gagal mengunduh '{}': status {}", nama, response.status());
        }

        let bytes = response.bytes().await?;

        let package_dir = modules_dir.join(&nama);
        let package_dir_clone = package_dir.clone();

        let has_rpl_json = tokio::task::spawn_blocking(move || -> Result<bool> {
            let cursor = Cursor::new(bytes);
            let mut zip = zip::ZipArchive::new(cursor)?;

            if !package_dir_clone.exists() {
                std_fs::create_dir_all(&package_dir_clone)?;
            }

            let mut has_json = false;

            for i in 0..zip.len() {
                let mut file = zip.by_index(i)?;
                let name = file.name().to_string();

                if name.ends_with(".rpl") || name.ends_with(".html") || name.ends_with("rpl.json") {
                    let mut parts: Vec<&str> = name.split('/').collect();
                    if parts.len() > 1 {
                        parts.remove(0);
                    }
                    let relative_path = parts.join("/");
                    if relative_path.is_empty() {
                        continue;
                    }

                    let target_path = package_dir_clone.join(&relative_path);
                    if let Some(p) = target_path.parent() {
                        std_fs::create_dir_all(p)?;
                    }

                    let mut out = std_fs::File::create(&target_path)?;
                    std::io::copy(&mut file, &mut out)?;

                    if relative_path == "rpl.json" {
                        has_json = true;
                    }
                }
            }
            Ok(has_json)
        })
        .await??;

        installed.insert(nama.clone());

        if has_rpl_json {
            let rpl_json_path = package_dir.join("rpl.json");
            let content = tokio::fs::read_to_string(&rpl_json_path).await?;
            if let Ok(config) = serde_json::from_str::<PaketConfig>(&content) {
                for (dep_nama, dep_url) in config.dependensi {
                    download_and_extract_recursive(
                        dep_nama,
                        dep_url,
                        modules_dir.clone(),
                        installed,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    })
}

pub fn hapus(paket: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let rpl_json_path = cwd.join("rpl.json");

    if !rpl_json_path.exists() {
        anyhow::bail!("File rpl.json tidak ditemukan. Jalankan 'rpl init' terlebih dahulu.");
    }

    let content = std::fs::read_to_string(&rpl_json_path)?;
    let mut config: PaketConfig = serde_json::from_str(&content)?;

    // Cari dan hapus dependensi berdasarkan nama
    let nama_paket = paket
        .split(':')
        .next_back()
        .unwrap_or(paket)
        .split('/')
        .next_back()
        .unwrap_or(paket);

    let ditemukan = config.dependensi.remove(nama_paket).is_some();

    if !ditemukan {
        // Coba cari berdasarkan value (url lengkap)
        let key_to_remove: Option<String> = config
            .dependensi
            .iter()
            .find(|(_, v)| v.as_str() == paket)
            .map(|(k, _)| k.clone());

        if let Some(key) = key_to_remove {
            config.dependensi.remove(&key);
        } else {
            println!(
                "\x1b[33m⚠️  Paket '{}' tidak ditemukan di rpl.json.\x1b[0m",
                paket
            );
            return Ok(());
        }
    }

    // Tulis ulang rpl.json
    let json_string = serde_json::to_string_pretty(&config)?;
    std::fs::write(&rpl_json_path, json_string)?;

    // Hapus folder dari rpl_modules
    let module_dir = cwd.join("rpl_modules").join(nama_paket);
    if module_dir.exists() {
        std::fs::remove_dir_all(&module_dir)?;
        println!("🗑️  Berhasil menghapus paket '{}'.", nama_paket);
    } else {
        println!(
            "🗑️  Paket '{}' dihapus dari rpl.json (folder tidak ditemukan).",
            nama_paket
        );
    }

    Ok(())
}

---
sidebar_position: 7
---

# Pustaka Standard (Standard Library)

Banyak bahasa pemrograman pemula hanya memungkinkan kamu mencetak teks ke layar. Namun, Rakoda tidak seperti itu! 

Rakoda dilengkapi dengan **Pustaka Standar** bawaan yang sangat kaya (*batteries-included*). Ini berarti, tanpa perlu menginstal paket tambahan yang rumit, kamu sudah bisa membuat File PDF atau bahkan memanggil Kecerdasan Buatan (AI).

Pustaka Standard Rakoda dikelompokkan ke dalam beberapa "Modul". Berikut adalah beberapa fungsi magis yang bisa langsung kamu gunakan:

## 1. Modul Dokumen (`dokumen`)

Membuat file PDF dari dalam kode hanya butuh 1 baris kode

```rakoda
buat isi_teks = "Halo dunia, ini adalah teks di dalam PDF pertama saya"

// Membuat file PDF bernama "halo_dunia.pdf"
dokumen.buat(isi_teks, "halo_dunia", "pdf")
```
Setelah dijalankan, sebuah file baru akan otomatis muncul di direktori proyek kamu.

## 2. Modul AI (`ai`)

Ingin membuat asisten pintar atau *chatbot* kamu sendiri? Gunakan modul AI bawaan Rakoda

```rakoda
// Tentukan penyedia AI (misalnya: deepseek)
ai.penyedia("deepseek")

// Masukkan API Key (kunci rahasia) kamu
ai.key("sk-kunci-rahasia-kamu")

// Bertanya langsung ke AI!
buat jawaban = ai.tanya("Siapakah penemu bola lampu?")
tampilkan jawaban
```

## 3. Modul Jaringan HTTP (`http`)

Aplikasi modern tidak pernah lepas dari internet. Rakoda memudahkan kamu mengambil data mentah (API) dari server lain di internet menggunakan jaringan HTTP.

```rakoda
buat respon = http.get("https://official-joke-api.appspot.com/random_joke")
tampilkan respon
```

## 4. Modul Web Server (`web`)

Satu keunggulan luar biasa Rakoda adalah kemampuannya menjadi pelayan internet (Web Server). kamu bisa membuat website kamu sendiri yang menyala selama 24 jam

```rakoda
fungsi halamanUtama()
    kembalikan "<h1>Halo dari Rakoda Server!</h1>"
selesai

web.get("/", halamanUtama)
web.jalankan(3000)
```
Buka *browser* kamu dan kunjungi `http://localhost:3000` untuk melihat hasilnya!

> Keren, bukan? Rakoda didesain agar kamu bisa langsung merasakan indahnya *Software Engineering* modern di jam pertama kamu belajar coding

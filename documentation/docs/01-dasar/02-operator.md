---
sidebar_position: 2
---

# Operator Dasar

Setelah kamu membuat Variabel, kamu tentu ingin melakukan perhitungan atau manipulasi terhadap nilai di dalamnya. Rakoda menyediakan kumpulan Operator yang sangat mudah diingat karena 100% menggunakan bahasa Indonesia untuk logika utamanya.

## 1. Operator Aritmatika

Seperti di pelajaran Matematika, Rakoda menggunakan simbol-simbol standar untuk perhitungan:

- **Tambah (`+`)**: `5 + 5`
- **Kurang (`-`)**: `10 - 2`
- **Kali (`*`)**: `3 * 4` (Menggunakan simbol bintang)
- **Bagi (`/`)**: `20 / 5` (Menggunakan simbol garis miring)
- **Modulus (`%`)**: `10 % 5` (Menggunakan simbol persentase)

```rakoda
buat apel = 10
buat jeruk = 5
buat total = apel + jeruk

tampilkan "Total buah: " + total
// Catatan: Tanda + juga bisa digunakan untuk menggabungkan dua Teks (String).
```

## 2. Operator Perbandingan

Operator ini digunakan untuk membandingkan dua nilai. Hasil akhirnya selalu berupa **Logika (Boolean)**, yaitu `benar` (true) atau `salah` (false). Sangat sering digunakan di dalam `jika` (Percabangan).

- **Sama Dengan (`===`) atau (`isinya`) atau (`hasilnya`)**: Digunakan untuk mengecek apakah dua nilai persis sama.
- **Lebih Besar (`>`) atau (`lebih dari`)**: Mengecek apakah nilai kiri lebih besar.
- **Lebih Kecil (`<`) atau (`kurang dari`)**: Mengecek apakah nilai kiri lebih kecil.

```rakoda
buat skor = 100
tampilkan skor == 100 // Hasil: benar
tampilkan skor > 50      // Hasil: benar
tampilkan skor < 10      // Hasil: salah
```

atau bisa juga

```rakoda
buat skor = 100
tampilkan skor isinya 100 // Hasil: benar
tampilkan skor lebih dari 50      // Hasil: benar
tampilkan skor kurang dari 10      // Hasil: salah
```

## 3. Operator Logika (Kata Hubung)

Ini adalah fitur yang paling unik di Rakoda. Alih-alih menggunakan simbol aneh seperti `&&` atau `||`, Anda cukup menulis kata bahasa Indonesia!

- **`dan` (AND)**: Akan bernilai `benar` jika **kedua syarat terpenuhi**.
- **`atau` (OR)**: Akan bernilai `benar` jika **salah satu syarat terpenuhi**.
- **`tidak` (NOT)**: Membalikkan keadaan (`benar` menjadi `salah`, dan sebaliknya).

```rakoda
buat nilai = 85
buat rajin = benar

jika nilai lebih dari 80 dan rajin isinya benar maka
    tampilkan "Kamu siswa berprestasi!"
selesai
```

Bagaimana? terasa seperti menulis kalimat biasa, bukan? kombinasikan ketiga jenis operator ini untuk merakit logika aplikasi serumit apa pun!

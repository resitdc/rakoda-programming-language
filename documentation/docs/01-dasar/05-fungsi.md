---
sidebar_position: 5
---

# Fungsi (Functions)

Ketika menulis program yang panjang, kita sering kali mendapati diri kita mengetik baris kode yang sama berulang-ulang di tempat yang berbeda. Hal ini membuat kode kita menjadi berantakan, sulit dibaca dan sulit diperbaiki jika ada kesalahan.

Untuk mengatasi ini, Rakoda memiliki sebuah fitur bernama **Fungsi**. 

Fungsi adalah sebuah blok kode yang dibungkus dan diberi nama, sehingga kamu dapat memanggilnya (menggunakannya) berkali-kali tanpa perlu menulis ulang kodenya.

## Membuat Fungsi Dasar

Untuk membuat fungsi, kita menggunakan kata `fungsi`, diikuti dengan nama fungsinya, parameter (opsional), dan ditutup dengan `maka` (opsional) dan `selesai`.

Berikut contoh fungsi untuk menyapa seseorang:

```rakoda
fungsi sapa(nama)
    tampilkan "Halo " + nama + ", selamat datang di Rakoda!"
selesai
```

Pada contoh di atas:
- `sapa` adalah **nama fungsi**.
- `nama` di dalam kurung `()` adalah **parameter**.

## Memanggil Fungsi

Fungsi yang sudah dibuat tidak akan melakukan apa-apa sampai kita "memanggilnya". Cara memanggilnya sangat mudah:

```rakoda
sapa("Restu")
sapa("Zidane")
sapa("Bernandus")
sapa("Icksan")
```

Hanya dengan 4 baris di atas, kita telah mencetak kalimat salam panjang sebanyak empat kali! Sangat praktis, bukan?

## Mengembalikan Nilai (`kembalikan`)

Sering kali, fungsi dibuat bukan untuk mencetak teks ke layar, melainkan untuk **melakukan perhitungan matematika** dan memberikan hasilnya kepada kita. 

Untuk memberikan (mengembalikan) hasil perhitungan, gunakan kata `kembalikan`.

```rakoda
// Membuat fungsi untuk menghitung luas persegi panjang
fungsi hitung_luas(panjang, lebar)
    buat luas = panjang * lebar
    kembalikan luas
selesai

// Menggunakan fungsi dan menyimpan hasilnya ke dalam variabel
buat hasil1 = hitung_luas(10, 5)
buat hasil2 = hitung_luas(20, 2)

tampilkan "Luas pertama adalah: " + hasil1
tampilkan "Luas kedua adalah: " + hasil2
```

> [!TIP] Kenapa harus pakai fungsi?
> Dengan membungkus kode rumus ke dalam sebuah `fungsi`, jika suatu hari ternyata rumus kamu salah, kamu hanya perlu memperbaikinya di **satu tempat** saja

## Kesimpulan

Fungsi adalah "pabrik kecil" di dalam program kamu. kamu memasukkan bahan mentah (parameter), pabrik tersebut akan memprosesnya (kode), dan akhirnya mengeluarkan barang jadi (nilai kembalian).

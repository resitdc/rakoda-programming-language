---
sidebar_position: 3
---

# Percabangan (Logika)

Percabangan adalah cara sebuah program mengambil keputusan. Dalam dunia nyata, kita sering berpikir: *"Jika hari ini hujan, maka saya akan membawa payung."*

Dalam bahasa pemrograman lain ini sering disebut *if-else*. Di Rakoda, konsep ini dibuat sedekat mungkin dengan cara kita berbicara dalam bahasa Indonesia.

## Menggunakan `jika` dan `maka`

Untuk membuat sebuah kondisi atau syarat dasar, gunakan kombinasi kata `jika`, `maka`, dan diakhiri dengan penutup `selesai`.

Berikut contohnya:

```rakoda
buat nilai = 80

jika nilai > 70 maka // ini bisa juga menggunakan `lebih dari`
  tampilkan "Selamat, Kamu Lulus"
selesai
```

Pada contoh di atas:
- Program akan mengecek apakah isi dari variabel `nilai` lebih besar dari `70`.
- Karena `80` memang lebih besar dari `70`, maka perintah di dalam blok tersebut akan dijalankan dan tulisan `"Selamat, Kamu Lulus"` akan ditampilkan.
- Kata kunci `selesai` **sangat penting** karena ia memberi tahu Rakoda bahwa blok pengecekan `jika` sudah berakhir.

## Menggunakan `jika tidak`

Lalu, bagaimana jika syaratnya tidak terpenuhi? Kita bisa menambahkan jalan keluar menggunakan `jika tidak` (setara dengan *else*). 

Mari kita coba dengan teks (*string*):

```rakoda
buat nama = "Zidane"

jika nama isinya "Restu" maka
  tampilkan "Kamu Ganteng"
jika tidak
  tampilkan "Kamu Siapa?"
selesai
```

Pada kode di atas:
- Program akan mengecek apakah `nama` menyimpan nilai `"Restu"`.
- Karena nama yang tersimpan adalah `"Zidane"`, maka syarat pertama gagal.
- Program akan otomatis melompat dan menjalankan apa yang ada di bawah blok `jika tidak`, sehingga mencetak `"Kamu Ganteng"`.

> **Catatan:** 
- Operator `isinya` adalah cara Rakoda untuk mengecek kesamaan dua teks/nilai (sama seperti `==` di bahasa pemrograman lain).
- Saat ini percabangan logika (branching) masih belum tersedia di Rakoda

## Kesimpulan

Kemampuan mengambil keputusan ini adalah jantung dari semua program atau aplikasi! Dengan menguasai struktur `jika - maka - jika tidak - selesai`, Anda sudah bisa membuat aplikasi interaktif, validasi kata sandi (*password*), hingga kecerdasan buatan (*AI*) sederhana.

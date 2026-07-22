---
sidebar_position: 2
---

# Percabangan (Logika)

Percabangan adalah cara sebuah program mengambil keputusan. Dalam dunia nyata, kita sering berpikir: *"Jika hari ini hujan, maka saya akan membawa payung."*

Di Rakoda, konsep ini (yang sering disebut *if-else*) dibuat sedekat mungkin dengan cara kita berbicara dalam bahasa Indonesia.

## Menggunakan `jika` dan `maka`

Untuk membuat sebuah kondisi atau syarat dasar, gunakan kombinasi kata kunci `jika`, `maka`, dan diakhiri dengan penutup `selesai`.

Berikut contohnya:

```rakoda
buat nilai = 80

jika nilai > 70 maka
  tampilkan "Selamat, Anda Lulus!"
selesai
```

Pada contoh di atas:
- Program akan mengecek apakah isi dari variabel `nilai` lebih besar dari `70`.
- Karena `80` memang lebih besar dari `70`, maka perintah di dalam blok tersebut akan dijalankan dan tulisan `"Selamat, Anda Lulus!"` akan ditampilkan.
- Kata kunci `selesai` **sangat penting** karena ia memberi tahu Rakoda bahwa blok pengecekan `jika` sudah berakhir.

## Menggunakan `jika tidak`

Lalu, bagaimana jika syaratnya tidak terpenuhi? Kita bisa menambahkan jalan keluar menggunakan `jika tidak` (setara dengan *else*). 

Mari kita coba dengan teks (*string*):

```rakoda
buat nama = "Budi"

jika nama isinya "Restu" maka
  tampilkan "Kamu Tampan!"
jika tidak
  tampilkan "Kamu Siapa?"
selesai
```

Pada kode di atas:
- Program akan mengecek apakah `nama` menyimpan nilai `"Restu"`.
- Karena nama yang tersimpan adalah `"Budi"`, maka syarat pertama gagal.
- Program akan otomatis melompat dan menjalankan apa yang ada di bawah blok `jika tidak`, sehingga mencetak `"Kamu Siapa?"`.

> **Catatan:** Operator `isinya` adalah cara Rakoda untuk mengecek kesamaan dua teks/nilai (sama seperti `==` di bahasa pemrograman lain).

## Menggunakan `jika tidak jika`

Untuk membuat keputusan yang memiliki banyak kemungkinan, kita bisa menyambungnya menggunakan kata kunci `jika tidak jika` (setara dengan *else if*).

```rakoda
buat poin = 50

jika poin > 80 maka
  tampilkan "Nilai Anda A"
jika tidak jika poin > 60 maka
  tampilkan "Nilai Anda B"
jika tidak
  tampilkan "Anda harus mengulang materi ini."
selesai
```

Dengan logika berlapis di atas:
- Rakoda akan mengecek dari baris paling atas terlebih dahulu. 
- Jika nilai `poin` lebih dari 80, ia berhenti di situ. 
- Jika gagal, ia akan mencoba mengecek syarat kedua (lebih dari 60). 
- Jika semua syarat tetap gagal, maka ia akan menjalankan baris di dalam `jika tidak` yang paling bawah.

## Kesimpulan

Kemampuan mengambil keputusan ini adalah jantung dari semua program atau aplikasi! Dengan menguasai struktur `jika - maka - jika tidak - selesai`, Anda sudah bisa membuat aplikasi interaktif, validasi kata sandi (*password*), hingga kecerdasan buatan (*AI*) sederhana.

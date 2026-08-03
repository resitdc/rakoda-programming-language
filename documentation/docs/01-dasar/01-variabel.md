---
sidebar_position: 1
---

# Variabel

Dalam bahasa pemrograman Rakoda, kamu dapat menyimpan data ke dalam sebuah "wadah" yang disebut sebagai variabel. 

Untuk membuat variabel, kita menggunakan kata `buat`.

## Membuat Variabel

Berikut adalah cara untuk membuat sebuah variabel di Rakoda:

```rakoda
buat nama = "Restu"
buat umur = 17
```

Pada contoh di atas:
- `nama` adalah sebuah variabel yang menyimpan tulisan atau teks (*string*) yaitu `"Restu"`.
- `umur` adalah sebuah variabel yang menyimpan angka bilangan bulat (*integer*) yaitu `17`.

## Menampilkan Variabel

Setelah variabel dibuat, kamu dapat menggunakannya atau menampilkan nilainya ke layar dengan perintah `tampilkan`.

```rakoda
buat nama = "Restu"
tampilkan nama
```

Ketika kode di atas dijalankan, program kamu akan mencetak teks `Restu` di layar terminal.

## Mengubah Isi Variabel

Variabel yang sudah dibuat bisa kita ubah lagi isinya di baris kode selanjutnya. kamu tidak perlu menggunakan kata `buat` lagi, cukup panggil nama variabelnya dan berikan nilai baru.

```rakoda
buat poin = 100
tampilkan poin

// Mengubah isi poin menjadi 200
poin = 200
tampilkan poin
```

## Aturan Penamaan Variabel

Ada beberapa aturan yang harus diikuti saat memberikan nama pada variabel kamu:
1. Nama variabel **tidak boleh mengandung spasi**.
2. Nama variabel tidak boleh diawali dengan angka.

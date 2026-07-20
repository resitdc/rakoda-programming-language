---
sidebar_position: 1
---

# Variabel

Dalam bahasa pemrograman Rakoda, Anda dapat menyimpan data ke dalam sebuah "wadah" yang disebut sebagai variabel. 

Untuk membuat variabel, kita menggunakan kata kunci `buat`.

## Membuat Variabel

Berikut adalah cara paling sederhana untuk membuat sebuah variabel di Rakoda:

```rakoda
buat nama = "Restu"
buat umur = 17
```

Pada contoh di atas:
- `nama` adalah sebuah variabel yang menyimpan tulisan atau teks (*string*) yaitu `"Restu"`.
- `umur` adalah sebuah variabel yang menyimpan angka bilangan bulat (*integer*) yaitu `17`.

## Menampilkan Variabel

Setelah variabel dibuat, Anda dapat menggunakannya atau menampilkan nilainya ke layar dengan perintah `tampilkan`.

```rakoda
buat nama = "Restu"
tampilkan nama
```

Ketika kode di atas dijalankan, program Anda akan mencetak teks `Restu` di layar terminal.

## Mengubah Isi Variabel

Variabel yang sudah dibuat bisa kita ubah lagi isinya di baris kode selanjutnya. Anda tidak perlu menggunakan kata kunci `buat` lagi, cukup panggil nama variabelnya dan berikan nilai baru.

```rakoda
buat poin = 100
tampilkan poin

// Mengubah isi poin menjadi 200
poin = 200
tampilkan poin
```

## Aturan Penamaan Variabel

Ada beberapa aturan yang harus diikuti saat memberikan nama pada variabel Anda:
1. Nama variabel **tidak boleh mengandung spasi**.
2. Nama variabel tidak boleh diawali dengan angka.
3. Sebaiknya gunakan awalan huruf kecil (misalnya: `namaLengkap` atau `nama_lengkap`).

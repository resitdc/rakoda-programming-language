---
sidebar_position: 6
---

# Struktur Data Kompleks

Sejauh ini, kamu telah belajar cara menyimpan angka atau teks ke dalam sebuah variabel. Namun, bagaimana jika kamu ingin menyimpan daftar nama siswa dalam satu kelas? Jika jumlah siswanya ada 30, apakah kamu akan membuat 30 variabel yang berbeda?

Tentu saja tidak kan? Rakoda menyediakan cara cerdas untuk membungkus banyak data sekaligus menggunakan **Array / List** dan **Objek**.

## 1. Array / List

Array atau List digunakan untuk menyimpan sekumpulan nilai secara berurutan. kamu bisa membayangkannya seperti sebuah lemari loker yang memiliki nomor urut.

Untuk membuat Array, kita menggunakan kurung siku `[ ]` dan memisahkan setiap isinya dengan koma.

```rakoda
buat angka = [10, 20, 30, 40, 50]
buat namaSiswa = ["Andi", "Budi", "Citra"]
```

### Mengakses Isi Array / List

Di dalam dunia pemrograman, urutan (indeks) pada Array **selalu dimulai dari 0**, bukan dari 1
- Elemen pertama ada di indeks `0`
- Elemen kedua ada di indeks `1`, dan seterusnya.

```rakoda
tampilkan "Siswa pertama adalah: " + namaSiswa[0]
tampilkan "Siswa kedua adalah: " + namaSiswa[1]
```

### Mengubah Isi Array / List
Rakoda juga menyediakan fungsi bawaan untuk memanipulasi (menambah, menghapus, atau menghitung panjang) sebuah  Array. 
Gunakan awalan `list.` pada fungsi-fungsi ini:

```rakoda
buat keranjang = ["Apel", "Jeruk"]

// Menambah data ke urutan paling belakang
keranjang = list.tambah(keranjang, "Anggur")

// Menghitung jumlah barang
tampilkan "Jumlah buah: " + list.panjang(keranjang)

// Menghapus data di indeks 0 (Apel akan hilang)
keranjang = list.hapus(keranjang, 0)
```

## 2. Objek

Berbeda dengan Array yang mengkamulkan "nomor urut", **Objek** menyimpan data secara berpasangan antara **Kunci (Key)** dan **Nilai (Value)**. Ini sangat berguna untuk menyimpan kumpulan data terstruktur, seperti profil seorang siswa.

Untuk membuat Objek, kita menggunakan kurung kurawal `{ }`.

```rakoda
buat profilSiswa = {
    "nama": "Budi",
    "umur": 17,
    "nilai": [80, 90, 100]
}
```

### Mengakses Isi Objek

kamu bisa mengambil informasi dari dalam Objek dengan dua cara: menggunakan titik (`.`) atau menggunakan kurung siku (`[]`). Keduanya sama-sama diperbolehkan.

```rakoda
tampilkan "Nama Siswa: " + profilSiswa.nama
tampilkan "Umur Siswa: " + profilSiswa["umur"]

// Mengambil nilai ujian pertama (menggabungkan Objek dan Array)
tampilkan "Nilai ujian pertama: " + profilSiswa.nilai[0]
```

## Gabungan Struktur Data (Tingkat Lanjut)

Dalam dunia industri *Software Engineering*, kamu akan sering menjumpai kasus di mana Array dan Objek digabung menjadi satu. Misalnya, sebuah Array yang berisi Objek:

```rakoda
buat daftar_buku = [
    { judul: "Laskar Pelangi", penulis: "Andrea Hirata" },
    { judul: "Bumi Manusia", penulis: "Pramoedya Ananta Toer" }
]

tampilkan "Buku kedua ditulis oleh: " + daftar_buku[1].penulis
```

Dengan menguasai Array dan Objek, kamu telah memiliki pondasi kuat untuk membangun struktur data serumit apa pun untuk aplikasi kamu

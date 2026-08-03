---
sidebar_position: 10
---

# Sistem dan Berkas

Kelompok modul ini (terdiri dari `file`, `json`, `env`, dan `dokumen`) memungkinkan kode kamu untuk berinteraksi langsung dengan Sistem Operasi dan Hardisk komputer!

## 1. Modul Berkas (`file`)

Memudahkan kamu membuat, membaca, atau menghapus file dan folder.

### `file.tulis(path, isi)`
Membuat file teks baru (atau menimpanya jika sudah ada).
```rakoda
file.tulis("catatan.txt", "Ini adalah baris pertama!")
```

### `file.baca(path)`
Membaca seluruh isi file teks dan mengembalikannya sebagai String.
```rakoda
buat isi = file.baca("catatan.txt")
tampilkan isi
```

### Operasi File Lainnya
- `file.ada(path)`: Mengembalikan `benar` jika file/folder ada.
- `file.hapus(path)`: Menghapus file secara permanen.

---

## 2. Modul JSON (`json`)

Dalam pembuatan website modern, pengiriman data sering menggunakan format universal bernama JSON. Rakoda bisa membacanya dengan sangat mudah

### `json.parse(teks_json)`
Mengubah teks (String) berformat JSON menjadi struktur data Rakoda (Objek).
```rakoda
buat teks = '{"nama": "Restu", "poin": 100}'
buat data = json.parse(teks)

tampilkan data.nama // Hasil: Restu
```

### `json.stringify(data_kamus)`
Mengubah Objek Rakoda kembali menjadi Teks (String).

---

## 3. Modul Lingkungan (`env`)

Sering kali kamu menyimpan kunci rahasia (seperti *password database*) di dalam file bernama `.env`. Modul ini bertugas membaca rahasia tersebut.

### `env.get(kunci)`
Membaca variabel lingkungan.
```rakoda
buat rahasia = env.get("KATA_SANDI_DB")
```

### `env.set(kunci, nilai)`
Membuat/mengatur variabel lingkungan baru saat program berjalan.

---

## 4. Modul Dokumen Lanjut (`dokumen`)

Ini adalah keajaiban Rakoda. Alih-alih membuat file `.txt` biasa, kamu bisa membuat file Dokumen PDF menggunakan format HTML ringan

### `dokumen.buat(teks_sumber, nama_file, jenis)`

Parameter `jenis` bisa berupa `"pdf"` atau `"html"`.
```rakoda
buat struktur = "<h1>Laporan Keuangan</h1> <p>Bulan ini sukses besar!</p>"

// Rakoda otomatis mengubah struktur ini menjadi desain PDF yang indah
dokumen.buat(struktur, "laporan_bulan_ini", "pdf")
```

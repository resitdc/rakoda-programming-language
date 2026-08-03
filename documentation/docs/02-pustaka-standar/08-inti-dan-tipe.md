---
sidebar_position: 8
---

# Fungsi Inti & Pengecekan Tipe

Modul Inti (*Core*) berisi fungsi-fungsi krusial yang sudah otomatis dimuat oleh Rakoda dan bisa langsung kamu panggil kapan saja, di mana saja, tanpa harus menggunakan awalan modul apa pun.

## Tampilan & Masukan

### `tampilkan(nilai)`
Mencetak teks, angka, atau nilai apa pun ke layar terminal. Jika kamu memberikan lebih dari satu parameter, teks tersebut akan digabungkan.
```rakoda
tampilkan "Hasil ujian: "
tampilkan 100
```

## Konversi Tipe Data

### `angka(nilai)`
Mengubah teks (String) yang berisi angka menjadi tipe data Angka (Integer).
```rakoda
buat teks_angka = "200"
buat angka_asli = angka(teks_angka)
```

## Analisis Data

### `.panjang(nilai)`
Menghitung jumlah karakter di dalam Teks, atau jumlah item di dalam Array/List.
```rakoda
buat teks = "Rakoda"
tampilkan string.panjang(teks) // Hasil: 6

buat daftar = [1, 2, 3]
tampilkan list.panjang(daftar) // Hasil: 3
```
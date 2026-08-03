---
sidebar_position: 9
---

# Matematika & Manipulasi Teks

Modul `matematika` dan `string` disediakan khusus untuk memudahkan kamu mengolah angka-angka rumit atau mengubah format kalimat.

## Modul Matematika (`matematika`)

Semua fungsi ini dipanggil dengan menggunakan awalan `matematika.`

### `matematika.bulatkan(angka)`
Membulatkan angka desimal ke bawah menjadi bilangan bulatkan (*floor*).
```rakoda
buat angka = 10.8
tampilkan matematika.bulatkan(angka) // Hasil: 10
```

## Modul Teks / String (`string`)

Memanipulasi teks adalah salah satu pekerjaan paling sering dalam *Software Engineering*. Awali pemanggilan fungsi dengan `string.`

### `string.kecil(teks)`
Mengubah semua huruf di dalam teks menjadi huruf kecil (lowercase).
```rakoda
tampilkan string.kecil("RAKODA") // Hasil: rakoda
```

### `string.besar(teks)`
Mengubah semua huruf menjadi kapital (uppercase).
```rakoda
tampilkan string.besar("indonesia") // Hasil: INDONESIA
```

### `string.ganti(teks_asli, target, pengganti)`
Mencari bagian dari teks (`target`) dan menggantinya dengan `pengganti`.
```rakoda
buat kalimat = "Saya suka makan ayam"
tampilkan string.ganti(kalimat, "ayam", "bebek")
// Hasil: Saya suka makan bebek
```
---
sidebar_position: 11
---

# Waktu dan Kriptografi

Di bagian ini, kita akan membahas dua fitur krusial untuk membuat program yang berjalan di dunia nyata: pencatatan Waktu (Time) dan pengamanan Data (Kriptografi).

## Modul Waktu (`waktu`)

### `waktu.sekarang()`
Mengembalikan teks yang berisi waktu (tanggal, jam, menit, detik) saat kode ini dieksekusi. Formatnya akan sesuai dengan standar internasional ISO 8601.
```rakoda
tampilkan waktu.sekarang() 
// Hasil: "2026-08-01T15:30:00Z"
```

## Modul Keamanan Data (`kripto`)

Saat kamu menyimpan kata sandi pengguna di dalam database, kamu **tidak boleh** menyimpannya dalam bentuk teks asli (misal: `"password123"`). Jika hacker meretas database kamu, mereka akan langsung mengetahui semuanya!

Oleh karena itu, gunakan teknik perombakan satu-arah (Hashing) yang disediakan modul `kripto`. Awali pemanggilan fungsi dengan `kripto.`

### Enkripsi Klasik Hashing
- `kripto.md5(teks)`: Menghasilkan kode unik pendek (tidak terlalu aman lagi untuk kata sandi, tapi bagus untuk verifikasi file).
- `kripto.sha256(teks)`: Menghasilkan kode acak panjang yang sangat aman (digunakan oleh sistem keamanan militer).

```rakoda
buat kata_sandi = "rahasia123"
buat sandi_aman = kripto.sha256(kata_sandi)

tampilkan sandi_aman
// Hasil: a25b13c... (huruf acak yang tidak bisa dibalikkan lagi!)
```
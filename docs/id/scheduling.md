# Penjadwalan

Anda dapat meminta Labalaba untuk memulai task secara otomatis pada jadwal berulang menggunakan ekspresi cron.

## Mengatur jadwal

1. Buka formulir task (buat task baru atau edit task yang sudah ada).
2. Pindah ke tab **Advanced**.
3. Isi kolom **Cron Schedule** dengan ekspresi yang valid (lihat format di bawah).
4. Simpan task.

Biarkan kolom ini kosong jika Anda ingin memulai task secara manual saja.

> **Catatan:** Semua waktu jadwal dievaluasi dalam **UTC**. Perhatikan hal ini jika Anda ingin task berjalan pada waktu lokal tertentu.

## Format ekspresi cron

Labalaba menggunakan format cron **6 kolom**. Ini berbeda dari format Unix 5 kolom klasik yang mungkin sudah Anda kenal.

> **Peringatan:** Teks placeholder yang ditampilkan pada kolom **Cron Schedule** (`0 */6 * * *`) hanyalah contoh 5 kolom. Anda **harus** menyertakan kolom **seconds** (detik) di awal. Selalu gunakan 6 kolom, atau jadwal Anda tidak akan berjalan sesuai harapan.

Urutan kolom:

| Posisi | Kolom         | Nilai yang diizinkan        |
|--------|---------------|-----------------------------|
| 1      | Second        | 0 – 59                      |
| 2      | Minute        | 0 – 59                      |
| 3      | Hour          | 0 – 23                      |
| 4      | Day of month  | 1 – 31                      |
| 5      | Month         | 1 – 12                      |
| 6      | Day of week   | 0 – 7 (0 dan 7 = Minggu)    |

Sintaks cron standar berlaku: `*` berarti "setiap nilai", `*/n` berarti "setiap n satuan", rentang seperti `1-5`, dan daftar seperti `1,3,5`.

## Contoh

| Ekspresi cron        | Arti                                        |
|----------------------|---------------------------------------------|
| `0 0 */6 * * *`      | Setiap 6 jam                                |
| `0 0 0 * * *`        | Setiap hari pada tengah malam (UTC)         |
| `*/30 * * * * *`     | Setiap 30 detik                             |
| `0 0 9 * * 1-5`      | Pukul 09:00 UTC setiap hari kerja (Sen–Jum) |
| `0 0 0 * * 0`        | Setiap Minggu pada tengah malam (UTC)       |

> **Tip:** Jika Anda ingin menjalankan sesuatu setiap hari pada waktu lokal tertentu, konversikan waktu lokal Anda ke UTC terlebih dahulu. Misalnya, 09:00 UTC+7 adalah `0 0 2 * * *` (02:00 UTC).

## Terkait

- [Membuat Task](./creating-tasks.md)
- [Auto-Restart saat Crash](./auto-restart.md)
- [Dependensi](./dependencies.md)
- [Pengaturan](./settings.md)
- [Kembali ke Beranda](./README.md)

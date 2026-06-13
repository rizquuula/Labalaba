# Penjadwalan

Anda dapat meminta Labalaba untuk memulai task secara otomatis pada jadwal berulang menggunakan ekspresi cron.

## Mengatur jadwal

1. Buka formulir task (buat task baru atau edit task yang sudah ada).
2. Pindah ke tab **Advanced**.
3. Isi kolom **Cron Schedule** dengan ekspresi yang valid (lihat format di bawah).
4. Simpan task.

Biarkan kolom ini kosong jika Anda ingin memulai task secara manual saja.

> **Catatan:** Semua waktu jadwal dievaluasi dalam **UTC**. Perhatikan hal ini jika Anda ingin task berjalan pada waktu lokal tertentu.

> **Penting:** Jadwal hanya akan berjalan selama **daemon aktif**. Daemon berjalan selama jendela aplikasi terbuka. Agar jadwal tetap berjalan setelah Anda menutup jendela, aktifkan **Launch at login** di Settings — lihat [Layanan Latar Belakang (Mode Daemon)](./background-service.md).

## Format ekspresi cron

Labalaba menggunakan format cron **5 kolom** standar — format yang sama digunakan oleh Unix cron dan sebagian besar alat cron.

Urutan kolom:

| Posisi | Kolom         | Nilai yang diizinkan     |
|--------|---------------|--------------------------|
| 1      | Minute        | 0 – 59                   |
| 2      | Hour          | 0 – 23                   |
| 3      | Day of month  | 1 – 31                   |
| 4      | Month         | 1 – 12                   |
| 5      | Day of week   | 0 – 6 (0 = Minggu)       |

Sintaks cron standar berlaku: `*` berarti "setiap nilai", `*/n` berarti "setiap n satuan", rentang seperti `1-5`, dan daftar seperti `1,3,5`.

## Contoh

| Ekspresi cron    | Arti                                         |
|------------------|----------------------------------------------|
| `0 */6 * * *`    | Setiap 6 jam                                 |
| `0 0 * * *`      | Setiap hari pada tengah malam (UTC)          |
| `*/30 * * * *`   | Setiap 30 menit                              |
| `0 9 * * 1-5`    | Pukul 09:00 UTC setiap hari kerja (Sen–Jum)  |
| `0 0 * * 0`      | Setiap Minggu pada tengah malam (UTC)        |

> **Tip:** Jika Anda ingin menjalankan sesuatu setiap hari pada waktu lokal tertentu, konversikan waktu lokal Anda ke UTC terlebih dahulu. Misalnya, 09:00 UTC+7 adalah `0 2 * * *` (02:00 UTC).

## Terkait

- [Membuat Task](./creating-tasks.md)
- [Auto-Restart saat Crash](./auto-restart.md)
- [Layanan Latar Belakang (Mode Daemon)](./background-service.md)
- [Dependensi](./dependencies.md)
- [Pengaturan](./settings.md)
- [Kembali ke Beranda](./README.md)

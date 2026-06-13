# Auto-Restart saat Crash

Labalaba dapat secara otomatis memulai ulang sebuah task ketika mengalami crash (mulai ulang otomatis), sehingga kegagalan sementara dapat pulih tanpa perlu tindakan apa pun dari Anda.

> **Catatan:** Auto-restart hanya berjalan selama **daemon aktif**. Daemon berjalan selama jendela aplikasi terbuka. Agar auto-restart tetap aktif setelah Anda menutup jendela, aktifkan **Launch at login** di Settings — lihat [Layanan Latar Belakang (Mode Daemon)](./background-service.md).

## Cara mengaktifkannya

1. Buka formulir task (buat task baru atau edit task yang sudah ada).
2. Pindah ke tab **Advanced**.
3. Centang kotak **Auto-restart on crash**.
4. Simpan task.

Setelah diaktifkan, label **AUTO-RESTART** akan muncul pada kartu task sebagai pengingat.

## Kapan auto-restart dipicu

Auto-restart hanya akan aktif **jika** proses keluar dengan kode keluar bukan nol — artinya benar-benar terjadi crash atau kesalahan saat keluar. Keluar secara normal (kode keluar 0) dianggap sebagai penghentian yang disengaja; auto-restart **tidak** akan berjalan lagi meskipun kotak centangnya aktif.

> **Catatan:** Jika program Anda keluar dengan kode 0 setelah menyelesaikan tugasnya (misalnya sebuah skrip yang berjalan sekali), Labalaba tidak akan memulai ulang. Ini adalah perilaku yang diharapkan.

## Jadwal backoff (jeda mundur)

Labalaba tidak langsung memulai ulang setiap saat. Ia menggunakan jeda mundur eksponensial untuk menghindari membebani program yang terus gagal:

| Percobaan | Tunggu sebelum memulai ulang |
|-----------|------------------------------|
| ke-1      | 3 detik                      |
| ke-2      | 6 detik                      |
| ke-3      | 12 detik                     |
| ke-4      | 24 detik                     |
| ke-5      | 48 detik                     |

Jeda dibatasi maksimal 60 detik, dan Labalaba akan melakukan **maksimal 5 kali percobaan ulang berturut-turut**. Jika kelima percobaan gagal, task ditandai **crashed** dan dibiarkan — Anda perlu menginvestigasi dan memulainya secara manual.

## Aturan reset 30 detik

Penghitung percobaan berturut-turut akan direset ke nol jika sebuah proses tetap berjalan selama setidaknya **30 detik**. Artinya, task yang sempat stabil cukup lama lalu kemudian crash akan memulai jeda mundur dari awal (3 detik), bukan dihitung dari kegagalan sebelumnya.

Tombol **Start** atau **Restart** secara manual juga akan mereset penghitung ini.

## Kapan menggunakan auto-restart

Auto-restart berguna untuk:

- Layanan latar belakang yang berjalan lama dan harus tetap aktif secara terus-menerus.
- Program yang sesekali crash akibat kesalahan sementara (gangguan jaringan, kunci file sementara, dan sebagainya).
- Task yang tidak dapat Anda pantau secara aktif.

## Catatan penting

- Auto-restart tidak berguna jika program Anda keluar secara normal (kode 0) — lihat penjelasan di atas.
- Setelah 5 kali percobaan gagal, task tetap dalam status **crashed**. Periksa log untuk mengetahui akar masalahnya sebelum memulai ulang secara manual.
- Menggabungkan auto-restart dengan task yang sangat singkat dapat menghabiskan 5 percobaan dengan cepat. Pertimbangkan apakah jadwal cron lebih sesuai untuk kasus tersebut.

## Terkait

- [Membuat Task](./creating-tasks.md)
- [Mengelola Task](./managing-tasks.md)
- [Penjadwalan](./scheduling.md)
- [Layanan Latar Belakang (Mode Daemon)](./background-service.md)
- [Log](./logs.md)
- [Pemecahan Masalah](./troubleshooting.md)
- [Kembali ke Beranda](./README.md)

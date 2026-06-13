# Pemecahan Masalah

Panduan praktis untuk masalah umum dan cara mengatasinya.

---

## Task tidak mau berjalan

**Kemungkinan penyebab dan solusinya:**

- **Path salah** — Buka formulir edit task dan periksa kembali **Executable / Script Path**. Salin-tempel path dari file manager untuk menghindari kesalahan ketik.
- **Izin tidak memadai** — Di macOS atau Linux, berkas mungkin tidak memiliki izin eksekusi. Buka terminal dan jalankan `chmod +x /path/to/your/program`, lalu coba lagi.
- **Working directory salah** — Jika program Anda mengharapkan berkas relatif terhadap lokasinya, pastikan **Working Directory** diatur ke folder yang benar. Biarkan kosong untuk menggunakan direktori aplikasi itu sendiri.

---

## Task langsung menampilkan "crashed"

Task menampilkan **crashed** saat keluar dengan kode non-zero (error).

- Periksa panel **Logs** untuk melihat pesan error yang dicetak program sebelum berhenti.
- Tinjau kembali **arguments** yang Anda berikan — kesalahan ketik atau flag wajib yang hilang sering menyebabkan program langsung keluar.
- Pastikan semua **environment variables** yang dibutuhkan program telah diatur dengan benar di bagian **Environment** pada task.

---

## Tidak ada log yang muncul

Tiga alasan yang paling umum:

1. **Task dijalankan sebagai Run as Admin (Windows)** — Saat sebuah task dielevasi melalui UAC, outputnya tidak dapat ditangkap oleh Labalaba. Panel log akan kosong. Ini adalah keterbatasan Windows. Lihat [Admin Elevation](./admin-elevation.md).
2. **Task dipulihkan setelah aplikasi di-restart** — Jika Labalaba ditutup lalu dibuka kembali, aplikasi dapat mendeteksi bahwa suatu proses masih berjalan berdasarkan PID-nya, tetapi tidak dapat mengambil ulang output yang sudah dicetak sebelum aplikasi dimulai. Output baru akan muncul seiring task terus berjalan.
3. **Task belum mencetak apa pun** — Beberapa program melakukan buffer pada output-nya atau hanya mencatat log setelah kejadian tertentu. Tunggu sebentar, atau periksa dokumentasi program itu sendiri.

---

## "Cannot connect to daemon" atau konflik port

Mesin internal menggunakan port lokal (default **27015**) untuk komunikasi internal. Jika program lain di komputer Anda sudah menggunakan port tersebut, Labalaba tidak dapat menjalankan mesinnya.

**Solusi:**

1. Buka **Settings** (gear icon).
2. Di bagian **Daemon**, ubah **Daemon Port** ke nilai lain (angka dari 1024 hingga 65535 yang belum dipakai).
3. Klik **Save Settings** dan restart Labalaba.

---

## Scheduled task tidak pernah berjalan

Field **Cron Schedule** menggunakan format **6-field** yang mencakup detik:

```
second  minute  hour  day-of-month  month  day-of-week
```

Placeholder yang ditampilkan di antarmuka memiliki 5 field (format yang umum), tetapi Labalaba membutuhkan versi 6-field. Misalnya, untuk menjalankan setiap hari pukul 09.00 UTC:

```
0 0 9 * * *
```

Semua waktu diinterpretasikan sebagai UTC. Lihat [Scheduling (Cron)](./scheduling.md) untuk referensi lengkap.

---

## Perubahan pada tasks.yaml atau settings.yaml yang diedit langsung tertimpa

Labalaba menulis ke berkas-berkas ini setiap kali Anda menyimpan perubahan di antarmuka. Jika aplikasi terbuka bersamaan saat Anda mengedit berkas, perubahan Anda mungkin tertimpa pada penyimpanan berikutnya.

**Solusi:** Selalu tutup Labalaba sebelum mengedit salah satu berkas secara langsung, lalu buka kembali. Lihat [Berkas Konfigurasi](./configuration-files.md).

---

## Pemeriksaan pembaruan gagal

Jika mengklik **Check for Updates Now** menghasilkan error:

- Pastikan Anda memiliki koneksi internet yang aktif.
- Periksa apakah firewall atau perangkat lunak keamanan Anda mengizinkan Labalaba untuk membuat permintaan HTTPS keluar ke `github.com`.
- Coba lagi nanti — GitHub mungkin sedang tidak tersedia sementara.

Anda selalu dapat mengunduh rilis terbaru secara manual dari `https://github.com/rizquuula/labalaba/releases`.

---

## Task terus restart lalu berhenti

Auto-restart hanya dipicu saat keluar dengan kode non-zero. Jika sebuah task terus crash dan restart, Labalaba akan memperlambat secara bertahap (3 → 6 → 12 → 24 → 48 detik, maksimum 60) dan berhenti setelah **5 kali percobaan gagal berturut-turut**, meninggalkan task dalam status **crashed**.

Penghitung akan direset secara otomatis jika task tetap berjalan selama minimal 30 detik. **Start** atau **Restart** secara manual juga akan mengosongkan penghitung.

Untuk memahami mengapa task terus crash, periksa log-nya untuk melihat output error. Lihat [Auto-restart](./auto-restart.md) untuk detail selengkapnya.

---

## Terkait

- [Auto-restart](./auto-restart.md)
- [Scheduling (Cron)](./scheduling.md)
- [Admin Elevation](./admin-elevation.md)
- [Berkas Konfigurasi](./configuration-files.md)
- [Pengaturan](./settings.md)
- [FAQ](./faq.md)
- [Kembali ke Beranda](./README.md)

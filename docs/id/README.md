# Labalaba — Panduan Pengguna

**Pengelola proses (process manager) lintas platform dengan antarmuka desktop bergaya glassmorphism.**

Labalaba memungkinkan Anda menjalankan, memantau, dan mengontrol program apa pun di komputer Anda — binari server, skrip shell, aplikasi Python, dan lainnya — semuanya dari satu jendela desktop. Definisikan setiap program satu kali sebagai sebuah **task**, lalu jalankan, hentikan, mulai ulang, dan pantau secara langsung. Task tersimpan otomatis, jadi selalu siap dijalankan dengan satu klik.

> 🌏 Prefer English? [Read the English version →](../en/README.md)

> **Catatan:** Antarmuka aplikasi Labalaba berbahasa Inggris. Pada dokumentasi ini, label tombol dan menu ditulis **dalam bahasa Inggris** (persis seperti yang Anda lihat di layar), dengan penjelasan dalam Bahasa Indonesia.

---

## Apa yang bisa dilakukan Labalaba?

| Fitur | Manfaatnya bagi Anda |
|---|---|
| 🚀 **Jalankan apa saja** | Luncurkan executable, skrip, atau aplikasi Python apa pun dengan argumen, variabel lingkungan, dan direktori kerja khusus |
| 📋 **Task tersimpan** | Setiap task diingat antar-restart — jalankan lagi kapan saja dengan satu klik |
| ▶️ **Kontrol siklus hidup** | Mulai, hentikan, dan mulai ulang task secara instan |
| 🔁 **Auto-restart** | Otomatis menghidupkan kembali task yang crash (dengan jeda mundur cerdas); tetap aktif saat jendela ditutup dalam mode daemon |
| ⏰ **Penjadwalan** | Jalankan task sesuai jadwal cron 5-kolom standar; tetap aktif saat jendela ditutup dalam mode daemon |
| 🔗 **Dependensi & jeda** | Atur urutan dan waktu mulai dengan jeda dan dependensi |
| 🛡️ **Elevasi admin** | Jalankan task dengan hak akses Administrator (Windows) |
| 📡 **Log langsung** | Pantau stdout/stderr mengalir dalam penampil log gaya terminal secara real-time |
| 📊 **Statistik langsung** | Lihat status, PID, CPU %, dan penggunaan memori tiap task |
| 🔔 **Notifikasi** | Dapatkan peringatan desktop saat task crash atau berhenti tak terduga |
| 🌓 **Tema Terang / Gelap** | Antarmuka glassmorphism yang rapi, sesuai selera Anda |
| ⬆️ **Pembaruan otomatis** | Dapatkan pemberitahuan saat versi baru tersedia |

---

## Peta dokumentasi

### 🏁 Memulai
1. [Instalasi](./installation.md) — Unduh dan pasang di Windows, macOS, atau Linux (atau build dari sumber)
2. [Memulai](./getting-started.md) — Buat dan jalankan task pertama Anda dalam tiga langkah

### 📦 Bekerja dengan task
3. [Membuat Task](./creating-tasks.md) — Referensi lengkap formulir task: setiap kolom dijelaskan
4. [Mengelola Task](./managing-tasks.md) — Status, mulai/hentikan/mulai ulang, mengedit, menghapus, pencarian, dan statistik langsung

### ⚙️ Fitur secara mendalam
5. [Auto-restart](./auto-restart.md) — Otomatis menjaga task yang crash tetap berjalan
6. [Penjadwalan (Cron)](./scheduling.md) — Jalankan task sesuai jadwal
7. [Layanan Latar Belakang (Mode Daemon)](./background-service.md) — Jaga jadwal dan auto-restart tetap berjalan setelah Anda menutup jendela
8. [Dependensi & Jeda Mulai](./dependencies.md) — Kendalikan urutan dan waktu mulai
9. [Elevasi Admin](./admin-elevation.md) — Jalankan task sebagai Administrator
10. [Melihat Log](./logs.md) — Penampil log langsung dan berkas log di disk

### 🔧 Konfigurasi & bantuan
11. [Pengaturan](./settings.md) — Setiap pengaturan dijelaskan, beserta tema
12. [Notifikasi & Pembaruan](./notifications-and-updates.md) — Peringatan desktop dan tetap up to date
13. [Berkas Konfigurasi](./configuration-files.md) — `tasks.yaml`, `settings.yaml`, dan direktori data
14. [Pemecahan Masalah](./troubleshooting.md) — Solusi untuk masalah umum
15. [Tanya Jawab (FAQ)](./faq.md) — Pertanyaan yang sering diajukan

---

## Tur 60 detik

1. **Buka Labalaba.** Daemon latar belakang menyala otomatis — tidak ada yang perlu dipasang atau dijalankan terpisah. Aplikasi hidup di system tray; menutup jendela menyembunyikannya, bukan menutup aplikasi.
2. **Klik "New Task"**, beri nama, lalu pilih program yang akan dijalankan (binari, skrip `.py`/`.sh`/`.ps1`, apa saja).
3. **Tekan ▶ Start.** Program Anda berjalan, dan keluarannya mengalir langsung ke penampil log.
4. Labalaba terus memantaunya — menampilkan **CPU**, **memori**, dan **status** — serta dapat memulai ulang otomatis jika crash.

Siap? Mulai dari [Instalasi →](./installation.md)

---

<div align="center">

Dibuat dengan ❤️ menggunakan Rust · [Laporkan bug](https://github.com/rizquuula/labalaba/issues) · [Minta fitur](https://github.com/rizquuula/labalaba/issues)

</div>

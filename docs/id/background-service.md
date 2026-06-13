# Layanan Latar Belakang (Mode Daemon)

Labalaba dibangun di atas mesin latar belakang yang disebut **daemon** — sebuah proses tersendiri yang melakukan semua pekerjaan nyata: menjalankan task, memantau crash, menjalankan jadwal cron, dan mengalirkan log. Jendela GUI adalah klien ringan yang terhubung ke daemon melalui koneksi HTTP/WebSocket lokal yang diamankan token di `127.0.0.1`.

Memahami dua mode jalannya daemon akan membantu Anda memutuskan cara terbaik mengonfigurasi Labalaba sesuai alur kerja Anda.

---

## Dua mode

| Mode | Cara mengaktifkan | Yang tetap berjalan saat jendela ditutup |
|---|---|---|
| **Mode daemon** | Settings → **Launch at login** ON | Semuanya: jadwal cron tetap berjalan, auto-restart tetap memantau, daemon terus berjalan |
| **Hanya sesi** *(bawaan)* | **Launch at login** OFF | Hanya **proses** task yang sudah berjalan; cron dan auto-restart berhenti bersama daemon |

> **Catatan:** Pada kedua mode, proses task yang sudah Anda jalankan adalah proses OS dengan PID masing-masing. Menghentikan daemon atau menutup jendela tidak membunuh proses tersebut — mereka tetap berjalan. Labalaba akan menemukan kembali proses tersebut saat berikutnya dibuka dengan memeriksa PID-nya.

---

## Cara mengaktifkan mode daemon

1. Buka panel **Settings** (ikon roda gigi di bilah atas).
2. Di bagian **Notifications**, alihkan **Launch at login** ke **On**.
3. Klik **Save Settings**.

Sejak saat itu, daemon akan mulai otomatis saat login dan tetap berjalan setelah Anda menutup jendela.

Untuk mematikannya, kembalikan toggle yang sama ke **Off** lalu simpan. Daemon tidak akan lagi mulai otomatis; daemon akan berhenti saat Anda keluar dari aplikasi.

### Yang terdaftar per OS

Tidak diperlukan akses administrator atau root — Labalaba mendaftarkan entri autostart **level pengguna** yang hanya berjalan untuk akun Anda dan berhenti saat Anda keluar dari sesi.

| OS | Mekanisme |
|---|---|
| **Linux** | Layanan pengguna `systemd` (`~/.config/systemd/user/`) |
| **macOS** | Login Item / LaunchAgent (`~/Library/LaunchAgents/`) |
| **Windows** | Entri registry startup di `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |

---

## System tray

Labalaba hidup di system tray Anda. Menutup jendela utama **tidak menutup aplikasi** — jendela disembunyikan sementara ikon tray tetap muncul. Ini memungkinkan daemon tetap berjalan tanpa jendela yang memenuhi layar.

- **Buka kembali jendela:** klik ikon tray.
- **Keluar sepenuhnya:** klik kanan ikon tray lalu pilih **Quit** (atau gunakan **Quit** dari menu tray). Ini menghentikan daemon dan menutup aplikasi.
- **Satu instance:** meluncurkan Labalaba kembali saat sudah berjalan (bahkan dengan jendela tersembunyi) akan memfokuskan jendela yang ada, bukan membuka salinan kedua.

---

## Yang bertahan dari berbagai kondisi

| Kejadian | Proses task yang sudah berjalan | Jadwal cron & auto-restart |
|---|---|---|
| Tutup jendela (mode daemon ON) | Berjalan | Berjalan |
| Tutup jendela (mode daemon OFF) | Berjalan | Berhenti |
| Keluar dari tray | Berjalan | Berhenti |
| Restart komputer (mode daemon ON) | Berhenti | Lanjut saat login berikutnya |
| Restart komputer (mode daemon OFF) | Berhenti | Berhenti |

> **Catatan:** Proses task tidak otomatis dimulai saat reboot terlepas dari mode yang dipilih. Untuk menjalankan task saat login, kombinasikan mode daemon dengan jadwal cron `@reboot` — atau gunakan mekanisme startup OS Anda sendiri.

---

## Terkait

- [Penjadwalan (Cron)](./scheduling.md)
- [Auto-Restart saat Crash](./auto-restart.md)
- [Pengaturan](./settings.md)
- [Berkas Konfigurasi](./configuration-files.md)
- [Kembali ke Beranda](./README.md)

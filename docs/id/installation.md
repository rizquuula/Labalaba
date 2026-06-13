# Instalasi

Jalankan Labalaba di komputer Anda — unduh paket yang sudah jadi atau build dari sumber.

---

## Persyaratan sistem

| Platform | Persyaratan minimum |
|---|---|
| Windows | Windows 10 atau lebih baru, x64 |
| Linux | x64, distribusi yang kompatibel dengan AppImage (glibc 2.17+) |
| macOS | macOS 11 (Big Sur) atau lebih baru — Intel atau Apple Silicon |

---

## Unduh rilis yang sudah jadi

Semua rilis diterbitkan di [halaman Labalaba GitHub Releases](https://github.com/rizquuula/labalaba/releases).

| Platform | Jenis berkas | Keterangan |
|---|---|---|
| Windows x64 | Installer `.msi` | Jalankan installer; tidak perlu alat tambahan |
| Linux x64 | `.AppImage` | Berkas portabel tunggal; tidak perlu instalasi |
| macOS Intel | `.dmg` | Seret ke Applications seperti biasa |
| macOS Apple Silicon (M1/M2/M3) | `.dmg` | Build terpisah, dioptimalkan untuk ARM |

---

## Instalasi per platform

### Windows

1. Unduh berkas `.msi` untuk versi Anda.
2. Klik dua kali berkas tersebut dan ikuti petunjuk installer.
3. Jalankan **Labalaba** dari menu Start atau pintasan di desktop.

> **Catatan:** Windows mungkin menampilkan peringatan SmartScreen untuk installer yang belum ditandatangani. Klik **More info** lalu **Run anyway** untuk melanjutkan.

### Linux

1. Unduh berkas `.AppImage`.
2. Jadikan berkas tersebut executable:
   ```bash
   chmod +x Labalaba_*.AppImage
   ```
3. Jalankan:
   ```bash
   ./Labalaba_*.AppImage
   ```

> **Tip:** Pindahkan AppImage ke `~/Applications/` (atau folder mana pun yang ada di `$PATH` Anda) dan buat pintasan desktop agar lebih mudah diakses.

### macOS

1. Unduh berkas `.dmg` yang sesuai dengan chip Anda (Intel atau Apple Silicon).
2. Buka berkas `.dmg` dan seret **Labalaba** ke folder **Applications**.
3. Buka Labalaba dari **Applications** atau Spotlight.

> **Catatan:** Saat pertama kali dibuka, macOS mungkin menampilkan pesan bahwa aplikasi "cannot be opened because the developer cannot be verified." Buka **System Settings → Privacy & Security** dan klik **Open Anyway**.

---

## Build dari sumber

Build dari sumber memungkinkan Anda menjalankan versi pengembangan terbaru atau membuat paket distribusi sendiri.

### Prasyarat

| Alat | Versi minimum |
|---|---|
| Rust (dengan Cargo) | 1.75 atau lebih baru |
| Node.js | 18 atau lebih baru |
| npm | disertakan bersama Node.js |

### Langkah-langkah

1. Clone repositori:
   ```bash
   git clone https://github.com/rizquuula/labalaba.git
   cd labalaba
   ```

2. Instal dependensi frontend:
   ```bash
   make install
   ```

3. Jalankan aplikasi dalam mode pengembangan (hot-reload):
   ```bash
   make dev
   ```

4. Untuk membangun installer/paket distribusi bagi OS Anda saat ini:
   ```bash
   cd gui && npm run tauri build
   ```
   Bundel yang telah selesai ditulis ke:
   ```
   gui/src-tauri/target/release/bundle/
   ```

---

## Peluncuran pertama

Ketika Anda membuka Labalaba untuk pertama kalinya, mesin di latar belakang akan berjalan **secara otomatis di dalam aplikasi** — tidak ada yang perlu diinstal, dikonfigurasi, atau dijalankan secara terpisah. Jendela utama langsung siap digunakan.

Jika Anda melihat **"Connecting to daemon…"** sejenak saat startup, itu normal; pesan tersebut hilang begitu mesin siap (biasanya kurang dari satu detik).

---

## Tempat data disimpan

Labalaba menyimpan berkas datanya di direktori kerja aplikasi secara default (di samping berkas biner saat produksi):

| Berkas | Isi |
|---|---|
| `tasks.yaml` | Semua task yang tersimpan |
| `settings.yaml` | Pengaturan aplikasi |
| `logs/` | Berkas log per task |

Anda dapat mengubah lokasi ini dengan menetapkan variabel lingkungan `LABALABA_DATA_DIR` sebelum menjalankan aplikasi. Lihat [Berkas Konfigurasi](./configuration-files.md) untuk detailnya.

---

## Langkah Berikutnya

- [Memulai](./getting-started.md) — Buat dan jalankan task pertama Anda dalam tiga langkah
- [Berkas Konfigurasi](./configuration-files.md) — Memahami `tasks.yaml`, `settings.yaml`, dan direktori data
- [Pemecahan Masalah](./troubleshooting.md) — Solusi untuk masalah instalasi yang umum

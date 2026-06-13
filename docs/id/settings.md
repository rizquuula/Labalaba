# Pengaturan

Sesuaikan tampilan dan perilaku Labalaba melalui panel **Settings** (pengaturan) — semua opsi tersedia di satu tempat.

## Membuka Pengaturan

Klik **gear icon** (ikon roda gigi) di bilah atas untuk membuka panel Settings.

## Tombol ganti tema

**sun/moon button** (tombol matahari/bulan) di bilah atas berfungsi untuk beralih seketika antara tema **Light** dan **Dark**. Pilihan Anda akan diingat antarsesi. Tema juga dapat diganti dari dalam Settings di bagian **Appearance**.

## Appearance

| Label | Default | Nilai | Fungsi |
|---|---|---|---|
| **Theme** | Dark | Dark, Light | Mengatur tema warna aplikasi. Langsung diterapkan saat dipilih. |

## Daemon

Pengaturan ini mengendalikan mesin internal yang menjalankan task-task Anda. Sebagian besar pengguna tidak perlu mengubahnya.

| Label | Default | Rentang | Fungsi |
|---|---|---|---|
| **Daemon Port** | 27015 | 1024–65535 | Port jaringan lokal yang digunakan oleh mesin internal secara internal. Ubah hanya jika port 27015 sudah dipakai oleh program lain. |
| **Config File Path** | `./tasks.yaml` | Path apa saja yang valid | Lokasi penyimpanan definisi task di disk. Path relatif diselesaikan terhadap direktori data. |
| **Log Buffer (lines)** | 5000 | 100–50000 | Jumlah maksimum baris log yang disimpan di memori per task untuk penampil live. Baris yang lebih lama akan dihapus saat batas tercapai. |

## Logs

| Label | Default | Rentang | Fungsi |
|---|---|---|---|
| **Log Directory** | `./logs` | Path apa saja yang valid | Folder tempat berkas log per task ditulis. Path relatif diselesaikan terhadap direktori data. |
| **Max File Size (MB)** | 10 | 1–1024 | Saat berkas log suatu task melebihi ukuran ini, berkas tersebut akan dirotasi (diganti nama) dan berkas baru dimulai. |
| **Max Rotated Files** | 5 | 0–100 | Jumlah berkas log lama (yang telah dirotasi) yang disimpan per task. Atur ke 0 agar tidak ada yang disimpan. |

## Notifications

| Label | Default | Nilai | Fungsi |
|---|---|---|---|
| **Desktop Notifications** | On | On / Off | Mengirim notifikasi desktop saat sebuah task crash atau berhenti secara tidak terduga. |
| **Launch at login** | Off | On / Off | Menjalankan Labalaba secara otomatis saat Anda masuk ke komputer. |

## Updates

| Label | Default | Fungsi |
|---|---|---|
| **Auto-check for Updates** | On | Memeriksa versi baru sekitar sekali sehari dan menampilkan dialog saat versi baru ditemukan. |
| **Check for Updates Now** | — (tombol) | Segera memeriksa versi terbaru. Menampilkan "Update available: X.Y.Z" beserta tautan **View Release**, atau "You're on the latest version (X.Y.Z)" jika Anda sudah menggunakan versi terkini. |

## Menyimpan perubahan

Input angka secara otomatis dibatasi pada rentang yang valid — Anda tidak dapat memasukkan nilai di luar rentang yang diizinkan. Setelah selesai, klik **Save Settings** (tombol akan sejenak menampilkan "Saving…" sebagai konfirmasi). Klik **Cancel** untuk membuang semua perubahan yang belum disimpan.

> **Tip:** Tutup semua task yang sedang berjalan dan menulis log sebelum mengubah **Log Directory** — berkas log untuk task yang aktif akan terus ditulis ke lokasi lama sampai task tersebut di-restart.

## Terkait

- [Notifications & Updates](./notifications-and-updates.md)
- [Berkas Konfigurasi](./configuration-files.md)
- [Pemecahan Masalah](./troubleshooting.md)
- [Kembali ke Beranda](./README.md)

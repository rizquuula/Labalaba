# Log

Labalaba merekam output setiap task dan memungkinkan Anda melihatnya secara langsung melalui penampil log bawaan, sekaligus menyimpannya di disk untuk diperiksa nanti.

## Membuka penampil log

Klik **View Logs** pada kartu task mana pun untuk membuka penampil log. Penampil ini muncul sebagai panel yang dapat diperluas dan diubah ukurannya (tinggi sekitar 180–380 px) yang muncul di bawah daftar task.

### Header penampil

Header penampil log berisi:

| Kontrol | Deskripsi |
|---------|-----------|
| Nama task | Menunjukkan task mana yang sedang Anda lihat. |
| Kotak centang **Auto-scroll** | Aktif secara default. Menjaga tampilan tetap bergulir ke output terbaru. Nonaktifkan untuk menggulir ke atas secara bebas. |
| Ikon **Clear** | Menghapus tampilan di layar. **Tidak** menghapus file log di disk. |
| **Close** (×) | Menutup panel log. |

### Yang Anda lihat saat pertama kali dibuka

Saat pertama kali membuka penampil log, ia memuat sekitar 500 baris terbaru dari file log di disk dan menampilkan "Loading historical logs…" selama proses tersebut. Setelah itu, output baru mengalir secara langsung.

Jika belum ada output, penampil menampilkan "Waiting for output…".

### Format baris

Setiap baris dalam penampil mencakup:

- Stempel waktu dalam format **HH:MM:SS**.
- Untuk output stderr: awalan **[ERR]** berwarna merah dan teks berwarna merah.
- Untuk output stdout: warna teks normal.

Penampil menggunakan font monospace. Baris yang panjang akan terbungkus, bukan meluap keluar.

> **Catatan:** Penampil menyimpan hingga sekitar 5.000 baris terbaru dalam memori. Baris yang lebih lama akan dihapus seiring masuknya baris baru. Riwayat lengkap selalu tersedia di file log di disk.

### Keterbatasan

- **Task yang dielevasi** (task dengan **Run as Admin** aktif di Windows) berjalan dalam sesi terpisah. Labalaba tidak dapat menangkap outputnya, sehingga penampil log akan kosong atau hanya menampilkan konten yang sangat terbatas untuk task tersebut.
- **Proses yang dipulihkan setelah aplikasi dimulai ulang** mungkin juga tidak menghasilkan output log langsung di penampil, meskipun output sebelumnya masih tersimpan di disk.

## File log di disk

Labalaba menulis output setiap task ke sebuah file log:

```
<data dir>/logs/<task-id>.log
```

Direktori data secara default adalah direktori kerja aplikasi dan dapat diganti dengan variabel lingkungan `LABALABA_DATA_DIR`.

### Format baris di disk

```
[timestamp] [stream] line
```

Contohnya:

```
[2026-06-13T09:00:00Z] [stdout] Server listening on port 8080
[2026-06-13T09:00:01Z] [stderr] Warning: config file not found, using defaults
```

### Rotasi log

Saat file log melebihi batas **Max File Size (MB)**, Labalaba melakukan rotasi:

| Nama file          | Isi                                       |
|--------------------|-------------------------------------------|
| `<task-id>.log`    | Output saat ini (paling baru)             |
| `<task-id>.log.1`  | Log sebelumnya (rotasi pertama)           |
| `<task-id>.log.2`  | Log lebih lama (rotasi kedua)             |
| …                  | …                                         |
| `<task-id>.log.5`  | Log paling lama yang disimpan (pengaturan default) |

Setelah jumlah file yang dirotasi mencapai maksimum (**Max Rotated Files**, default 5), file paling lama akan dihapus saat rotasi berikutnya terjadi.

### Pengaturan terkait

Pengaturan berikut pada halaman **Settings** mengontrol perilaku log:

| Pengaturan | Default | Deskripsi |
|------------|---------|-----------|
| Log Directory | `<data dir>/logs` | Lokasi penyimpanan file log. |
| Max File Size (MB) | 10 MB | Ukuran file yang memicu rotasi. |
| Max Rotated Files | 5 | Jumlah file yang dirotasi untuk disimpan. |
| Log Buffer (lines) | — | Jumlah baris yang di-buffer dalam memori. |

> **Tip:** Jika ruang disk menjadi perhatian, turunkan nilai **Max File Size (MB)** dan **Max Rotated Files** di Settings.

## Terkait

- [Elevasi Admin](./admin-elevation.md)
- [Mengelola Task](./managing-tasks.md)
- [Pengaturan](./settings.md)
- [File Konfigurasi](./configuration-files.md)
- [Pemecahan Masalah](./troubleshooting.md)
- [Kembali ke Beranda](./README.md)

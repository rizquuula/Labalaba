# Berkas Konfigurasi

Labalaba menyimpan semua yang dibutuhkan — task, pengaturan, dan berkas log — dalam satu folder yang disebut **direktori data**.

---

## Direktori data

Labalaba menentukan direktori data dengan urutan berikut, berhenti pada kecocokan pertama:

1. **Variabel lingkungan `LABALABA_DATA_DIR`** — jika diatur dan tidak kosong, digunakan apa adanya.
2. **Mode portable** — jika diaktifkan, `<folder tempat berkas executable berada>/data`. Lihat [Mode portable](#mode-portable) di bawah. Hanya untuk Windows.
3. **Direktori data per-pengguna bawaan platform** + `labalaba` — inilah nilai default pada instalasi normal:
   - Windows: `%APPDATA%\labalaba`
   - Linux: `~/.local/share/labalaba`
   - macOS: `~/Library/Application Support/labalaba`
4. `.` (direktori kerja saat ini) — hanya jika direktori data platform tidak dapat ditentukan; tidak diharapkan terjadi pada instalasi normal.

Direktori data **bukan** direktori kerja aplikasi dan **bukan** direktori instalasi. Semua berkas berada di sana:

| Item | Path default | Isi |
|---|---|---|
| `tasks.yaml` | `<direktori data>/tasks.yaml` | Semua definisi task Anda |
| `settings.yaml` | `<direktori data>/settings.yaml` | Semua pengaturan aplikasi |
| `logs/` | `<direktori data>/logs/` | Berkas log per task |

### Mengubah direktori data

Atur variabel lingkungan `LABALABA_DATA_DIR` sebelum menjalankan Labalaba untuk mengarahkan aplikasi ke folder lain:

```
LABALABA_DATA_DIR=/home/you/labalaba-data
```

Semua path relatif di dalam `settings.yaml` (seperti `./tasks.yaml` atau `./logs`) diselesaikan terhadap direktori ini, bukan terhadap direktori kerja saat ini.

> **Tip:** Ini berguna jika Anda ingin menyimpan data di drive bersama atau folder pengguna tertentu.

### Mode portable

Mode portable menyimpan seluruh data Labalaba di sebelah aplikasi, bukan di profil per-pengguna Anda, sehingga aplikasi dan datanya berada dalam satu folder — memudahkan pencadangan dan menjaga semuanya di satu drive. Mode ini bersifat **opt-in** dan **hanya untuk Windows**: folder instalasi default (`C:\Program Files\Labalaba`) tidak dapat ditulis tanpa elevasi selagi daemon berjalan tanpa elevasi, sehingga tidak ada yang berpindah ke sana secara otomatis. Mode ini tidak tersedia di macOS (menulis di dalam `Labalaba.app` merusak tanda tangan kode/code signature dan akan terhapus saat pembaruan berikutnya) maupun Linux (AppImage terpasang read-only di path sementara yang baru setiap kali dijalankan, dan instalasi `.deb` menempati `/usr/bin` milik root).

Aktifkan lewat **Settings → Data Location**, yang juga menampilkan direktori data yang sedang aktif dan memiliki tombol **Reveal** untuk membuka folder tersebut di file manager. Saat toggle diubah:

1. Daemon dihentikan.
2. `tasks.yaml`, `settings.yaml`, dan `logs/` disalin ke `<folder tempat berkas executable berada>/data`. Ini adalah **penyalinan**, bukan pemindahan, dan tidak pernah menimpa berkas yang sudah ada di tujuan — jika `tasks.yaml` sudah ada di sana, berkas itulah yang akan dimuat, bukan hasil salinan dari sumbernya. Dialog konfirmasi menjelaskan hal ini sebelum Anda melanjutkan.
3. Berkas penanda `labalaba.portable` ditulis (atau, saat beralih kembali, dihapus) di sebelah berkas executable — keberadaannya yang mengaktifkan mode ini.
4. Daemon dijalankan ulang dengan direktori yang baru.

Task yang sedang berjalan tetap bertahan saat beralih, dan data asli dibiarkan apa adanya sebagai cadangan — tidak ada yang dihapus saat Anda mengaktifkan atau menonaktifkan mode portable.

> **Catatan:** Nilai `config_path` / `log_dir` yang absolut di `settings.yaml` tidak pernah disentuh oleh peralihan mode portable — lihat bagian [settings.yaml](#settingsyaml) di bawah.

---

## tasks.yaml

Berkas ini menyimpan setiap task yang telah Anda buat. Berkas ini dibaca saat Labalaba memulai dan diperbarui setiap kali Anda membuat, mengedit, atau menghapus task melalui antarmuka.

### Contoh

```yaml
tasks:
  - id: "550e8400-e29b-41d4-a716-446655440000"
    description: "My API Server"
    executable: "C:\\Apps\\server.exe"
    arguments: ["--port", "8080"]
    working_directory: "C:\\Apps"
    environment:
      NODE_ENV: "production"
    run_as_admin: false
    auto_restart: true
    schedule: null
    startup_delay_ms: 0
    depends_on: []
    runner_prefix: null
    pids: []
```

### Referensi field

| Field | Isi | Catatan |
|---|---|---|
| `id` | Pengenal unik untuk task | Dibuat secara otomatis — jangan ubah nilai ini |
| `description` | Nama tampilan yang ditunjukkan di antarmuka | |
| `executable` | Path lengkap ke program atau skrip | |
| `arguments` | Daftar argumen baris perintah | |
| `working_directory` | Folder tempat task berjalan | Kosongkan untuk menggunakan direktori kerja aplikasi |
| `environment` | Peta key/value untuk variabel lingkungan | |
| `run_as_admin` | `true` untuk elevasi (Windows UAC) | `false` di macOS/Linux (tidak berpengaruh) |
| `auto_restart` | `true` untuk restart saat keluar secara tidak terduga | |
| `schedule` | String cron 6-field atau `null` | Lihat [Scheduling](./scheduling.md) |
| `startup_delay_ms` | Milidetik tunggu sebelum memulai | Berguna bersama `depends_on` |
| `depends_on` | Daftar nilai `id` task yang harus dimulai lebih dulu | Lihat [Dependencies](./dependencies.md) |
| `runner_prefix` | Prefix interpreter, misalnya `"uv run"` | `null` untuk menjalankan langsung |
| `pids` | ID proses dari task yang sedang berjalan | Dikelola otomatis — biarkan sebagai `[]` |

> **Catatan:** Field `depends_on` hanya dapat diatur dengan mengedit `tasks.yaml` secara langsung — belum ada antarmuka untuk itu. Lihat [Dependencies & Startup Delay](./dependencies.md) untuk detail selengkapnya.

---

## settings.yaml

Berkas ini menyimpan semua preferensi aplikasi Anda. Berkas ini diperbarui setiap kali Anda mengklik **Save Settings** di antarmuka.

### Contoh (menampilkan nilai default)

```yaml
theme: "dark"
daemon_port: 27015
log_buffer_lines: 5000
config_path: "./tasks.yaml"
notifications_enabled: true
auto_check_updates: true
update_check_interval_hours: 24
launch_on_startup: false
log_dir: "./logs"
log_max_file_size_mb: 10
log_max_rotated_files: 5
```

### Referensi field

| Field | Default | Fungsi |
|---|---|---|
| `theme` | `"dark"` | `"dark"` atau `"light"` |
| `daemon_port` | `27015` | Port mesin internal (1024–65535) |
| `log_buffer_lines` | `5000` | Maks baris log di memori per task (100–50000) |
| `config_path` | `"./tasks.yaml"` | Path ke berkas definisi task. Path relatif diselesaikan terhadap direktori data, bukan direktori kerja; path absolut digunakan apa adanya |
| `notifications_enabled` | `true` | Notifikasi desktop saat crash/berhenti, aktif atau tidak |
| `auto_check_updates` | `true` | Periksa pembaruan sekali sehari |
| `update_check_interval_hours` | `24` | Jam antara pemeriksaan pembaruan otomatis |
| `launch_on_startup` | `false` | Jalankan Labalaba saat Anda masuk |
| `log_dir` | `"./logs"` | Folder untuk berkas log per task. Path relatif diselesaikan terhadap direktori data, bukan direktori kerja; path absolut digunakan apa adanya |
| `log_max_file_size_mb` | `10` | Rotasi berkas log setelah mencapai ukuran ini (MB) |
| `log_max_rotated_files` | `5` | Berkas log lama yang disimpan per task (0 = tidak ada) |

> **Catatan:** `config_path` dan `log_dir` tidak ikut berpindah saat Anda mengaktifkan atau menonaktifkan mode portable — jika salah satunya Anda arahkan ke path absolut, itu adalah penempatan yang disengaja dan tidak akan dipindahkan oleh mode portable. Hanya path relatif (yang menurut definisinya berada di dalam direktori data) yang ikut berpindah.

---

## Mengedit berkas secara langsung

Kedua berkas adalah YAML biasa dan dapat dibuka di editor teks apa pun. Ini adalah satu-satunya cara untuk mengatur beberapa opsi lanjutan (seperti `depends_on` di `tasks.yaml`).

> **Peringatan:** Selalu tutup Labalaba sebelum mengedit `tasks.yaml` atau `settings.yaml`. Jika aplikasi sedang berjalan, perubahan Anda mungkin ditimpa saat aplikasi menyimpan berikutnya.

> **Tip:** Sebelum melakukan perubahan besar, salin kedua berkas ke lokasi yang aman sebagai cadangan. Untuk memindahkan seluruh konfigurasi ke komputer lain, salin `tasks.yaml` dan `settings.yaml` ke lokasi relatif yang sama di mesin baru.

---

## Terkait

- [Pengaturan](./settings.md)
- [Dependencies & Startup Delay](./dependencies.md)
- [Scheduling (Cron)](./scheduling.md)
- [Pemecahan Masalah](./troubleshooting.md)
- [Kembali ke Beranda](./README.md)

# Berkas Konfigurasi

Labalaba menyimpan semua yang dibutuhkan — task, pengaturan, dan berkas log — dalam satu folder yang disebut **direktori data**.

---

## Direktori data

Secara default, direktori data adalah direktori kerja aplikasi (folder yang sama dengan berkas binary Labalaba pada instalasi normal). Semua berkas berada di sana:

| Item | Path default | Isi |
|---|---|---|
| `tasks.yaml` | `./tasks.yaml` | Semua definisi task Anda |
| `settings.yaml` | `./settings.yaml` | Semua pengaturan aplikasi |
| `logs/` | `./logs/` | Berkas log per task |

### Mengubah direktori data

Atur variabel lingkungan `LABALABA_DATA_DIR` sebelum menjalankan Labalaba untuk mengarahkan aplikasi ke folder lain:

```
LABALABA_DATA_DIR=/home/you/labalaba-data
```

Semua path relatif di dalam `settings.yaml` (seperti `./tasks.yaml` atau `./logs`) diselesaikan terhadap direktori ini.

> **Tip:** Ini berguna jika Anda ingin menyimpan data di drive bersama atau folder pengguna tertentu.

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
| `config_path` | `"./tasks.yaml"` | Path ke berkas definisi task |
| `notifications_enabled` | `true` | Notifikasi desktop saat crash/berhenti, aktif atau tidak |
| `auto_check_updates` | `true` | Periksa pembaruan sekali sehari |
| `update_check_interval_hours` | `24` | Jam antara pemeriksaan pembaruan otomatis |
| `launch_on_startup` | `false` | Jalankan Labalaba saat Anda masuk |
| `log_dir` | `"./logs"` | Folder untuk berkas log per task |
| `log_max_file_size_mb` | `10` | Rotasi berkas log setelah mencapai ukuran ini (MB) |
| `log_max_rotated_files` | `5` | Berkas log lama yang disimpan per task (0 = tidak ada) |

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

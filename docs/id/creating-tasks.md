# Membuat Task

Referensi lengkap untuk setiap kolom di formulir task — dari dasar hingga penjadwalan lanjutan.

---

## Membuka formulir

Klik tombol **New Task** di bagian atas daftar task. Jika Anda belum memiliki task, klik **Add your first task**. Formulir **New Task** akan terbuka sebagai overlay.

Formulir memiliki dua tab: **Basic** dan **Advanced**. Beralih antar tab dengan mengklik label tab atau menekan tombol panah **Left** / **Right**. Tekan **Escape** kapan saja untuk membatalkan tanpa menyimpan.

---

## Tab Basic

### Description (wajib diisi)

Nama tampilan yang ditampilkan di kartu task di seluruh Labalaba.

- Placeholder: `My Application`
- Tidak boleh kosong — formulir tidak dapat dikirim tanpa kolom ini.

### Executable / Script Path (wajib diisi)

Path ke program atau skrip yang ingin Anda jalankan.

- Placeholder: `Pick a binary, .py, .sh, .ps1, .bat…`
- Klik **Browse** untuk membuka pemilih berkas:
  - **Windows:** memfilter ke `.exe`, `.bat`, `.cmd`, `.ps1`, `.py`, `.pyw`
  - **macOS / Linux:** menampilkan semua berkas (sehingga berkas executable tanpa ekstensi dapat dipilih)
- Saat Anda memilih skrip (`.sh`, `.bat`, `.ps1`) melalui Browse, Labalaba **mendeteksi interpreter-nya secara otomatis** dan mengisi runner serta argumen untuk Anda.

### Python Runner

Dropdown ini muncul **hanya** ketika path berakhiran `.py` atau `.pyw`.

> **Catatan:** Petunjuk yang ditampilkan: "Detected a Python script — it will be launched via this runner."

| Pilihan | Yang dijalankan |
|---|---|
| `python` | Perintah `python` sistem |
| `pythonw` | pythonw Windows (tanpa jendela konsol) |
| `uv run` | Jalankan melalui manajer paket [uv](https://github.com/astral-sh/uv) |
| `pipenv run python` | Jalankan di dalam virtual environment Pipenv |
| `poetry run python` | Jalankan di dalam virtual environment Poetry |
| `custom…` | Menampilkan kolom **Custom Runner Command** — ketik perintah apa pun (misalnya, `uv run` atau `/home/user/.venv/bin/python`) |

### Arguments

Argumen baris perintah yang dipisahkan spasi dan diteruskan ke program Anda.

- Placeholder: `--port 8080 --config config.yaml`
- Argumen dipisah berdasarkan spasi saat task disimpan.

---

## Tab Advanced

### Working Directory

Folder tempat program dijalankan. Biarkan kosong untuk menggunakan direktori yang berisi berkas executable.

- Placeholder: `C:\path\to\workdir`
- Klik **Browse** untuk memilih folder.

### Environment Variables

Variabel lingkungan tambahan yang diteruskan ke proses. Satu pasang `KEY=VALUE` per baris.

```
NODE_ENV=production
PORT=8080
DATABASE_URL=postgres://localhost/myapp
```

> **Tip:** Nilai boleh mengandung `=` — hanya `=` **pertama** pada baris yang dianggap sebagai pemisah. Baris tanpa `=` diabaikan. Spasi di awal/akhir akan dihapus.

### Cron Schedule

Jalankan task secara otomatis sesuai jadwal. Menggunakan ekspresi cron 6 kolom.

- Placeholder: `0 */6 * * *` (opsional)
- Contoh: `0 9 * * 1-5` — setiap hari kerja pukul 09:00

Lihat [Penjadwalan (Cron)](./scheduling.md) untuk referensi format lengkap.

### Startup Delay (ms)

Berapa milidetik Labalaba menunggu setelah menerima perintah mulai sebelum benar-benar menjalankan proses.

- Minimum: `0`
- Contoh: `5000` = 5 detik

Berguna ketika suatu task bergantung pada layanan lain yang membutuhkan waktu untuk mulai. Lihat [Dependensi & Jeda Mulai](./dependencies.md).

### Run as Admin

Centang kotak ini untuk menjalankan task dengan hak istimewa Administrator (yang ditinggikan).

> **Peringatan:** Aktifkan ini hanya untuk program yang benar-benar memerlukan akses yang ditinggikan. Lihat [Elevasi Admin](./admin-elevation.md).

### Auto-restart on crash

Centang kotak ini agar Labalaba secara otomatis me-restart task jika keluar dengan kode error non-zero.

> **Catatan:** Keluar dengan bersih (exit code 0) **tidak** dianggap sebagai crash — auto-restart tidak akan terpicu. Lihat [Auto-restart](./auto-restart.md).

---

## Mengirim formulir

| Tombol | Ditampilkan saat | Label saat memuat |
|---|---|---|
| **Create Task** | Membuat task baru | "Creating…" |
| **Save Changes** | Mengedit task yang sudah ada | "Saving…" |
| **Cancel** | Selalu | — |

Jika validasi gagal (misalnya, **Description** atau **Executable / Script Path** kosong), atau terjadi error saat menyimpan, pesan error berwarna merah akan muncul di atas tombol footer.

---

## Cara kerja runner prefix

Ketika Anda menetapkan **Python Runner** (atau Labalaba mengisinya secara otomatis untuk sebuah skrip), runner tersebut menjadi "runner prefix". Kata pertama dari runner adalah perintahnya; sisanya menjadi argumen awal. Berkas executable dan argumen Anda sendiri menyusul setelahnya.

| Runner | Executable | Arguments | Yang sebenarnya dijalankan |
|---|---|---|---|
| `uv run` | `script.py` | `--port 8080` | `uv run script.py --port 8080` |
| `python` | `app.py` | `--verbose` | `python app.py --verbose` |
| `pipenv run python` | `main.py` | (tidak ada) | `pipenv run python main.py` |
| `node` | `server.js` | `--inspect` | `node server.js --inspect` |
| (tidak ada) | `bash` | `run.sh` | `bash run.sh` |

Dropdown **Python Runner** adalah pintasan yang nyaman untuk berkas `.py`. Untuk jenis skrip lain, fitur auto-fill dari **Browse** akan menanganinya. Anda juga dapat mengetikkan runner secara langsung ke kolom **Custom Runner Command**.

---

## Referensi kolom

### Tab Basic

| Kolom | Wajib | Tipe | Keterangan |
|---|---|---|---|
| **Description** | Ya | Teks | Nama tampilan; tidak boleh kosong |
| **Executable / Script Path** | Ya | Path berkas | Browse tersedia; mendeteksi runner untuk skrip secara otomatis |
| **Python Runner** | — | Dropdown | Muncul hanya untuk berkas `.py`/`.pyw` |
| **Custom Runner Command** | — | Teks | Muncul saat **Python Runner** = `custom…` |
| **Arguments** | Tidak | Teks | Dipisahkan spasi; dipisah saat disimpan |

### Tab Advanced

| Kolom | Wajib | Tipe | Keterangan |
|---|---|---|---|
| **Working Directory** | Tidak | Path folder | Browse tersedia; default ke direktori executable |
| **Environment Variables** | Tidak | Textarea | Satu `KEY=VALUE` per baris |
| **Cron Schedule** | Tidak | Teks | Ekspresi cron 6 kolom; lihat [Penjadwalan](./scheduling.md) |
| **Startup Delay (ms)** | Tidak | Angka (≥ 0) | Milidetik untuk menunggu sebelum dijalankan |
| **Run as Admin** | Tidak | Checkbox | Meningkatkan hak istimewa; lihat [Elevasi Admin](./admin-elevation.md) |
| **Auto-restart on crash** | Tidak | Checkbox | Restart saat keluar non-zero; lihat [Auto-restart](./auto-restart.md) |

---

## Contoh penggunaan

### Contoh 1 — Web server Node.js

| Kolom | Nilai |
|---|---|
| **Description** | `API Server` |
| **Executable / Script Path** | `/home/user/myapp/server.js` |
| **Arguments** | `--port 3000` |
| **Working Directory** | `/home/user/myapp` |
| **Environment Variables** | `NODE_ENV=production` |

Pada kolom **Arguments**, atur runner dengan mengetikkan `node` di **Custom Runner Command** (pilih `custom…` dari dropdown **Python Runner**… namun perlu diingat — Python Runner hanya muncul untuk berkas `.py`). Untuk berkas `.js`, masukkan `node` sebagai **Executable / Script Path** dan `server.js --port 3000` sebagai **Arguments**, atau pertahankan path sebagai `server.js` dan tetapkan custom runner `node`.

> **Tip:** Pendekatan paling sederhana untuk skrip Node adalah: **Executable / Script Path** = path lengkap ke `node` (misalnya, `/usr/bin/node`) dan **Arguments** = `server.js --port 3000`, dengan **Working Directory** mengarah ke folder proyek Anda.

### Contoh 2 — Aplikasi Python via `uv run`

| Kolom | Nilai |
|---|---|
| **Description** | `Data Pipeline` |
| **Executable / Script Path** | `/home/user/pipeline/main.py` |
| **Python Runner** | `uv run` |
| **Arguments** | `--env production` |
| **Working Directory** | `/home/user/pipeline` |
| **Auto-restart on crash** | Dicentang |

Labalaba akan menjalankan: `uv run /home/user/pipeline/main.py --env production`

### Contoh 3 — Shell script di Linux

| Kolom | Nilai |
|---|---|
| **Description** | `Backup Script` |
| **Executable / Script Path** | `/home/user/scripts/backup.sh` (dipilih melalui **Browse**) |
| **Cron Schedule** | `0 2 * * *` (setiap hari pukul 02:00) |

Menelusuri ke `backup.sh` akan mendeteksi `bash` sebagai runner secara otomatis dan mengisinya untuk Anda.

---

## Terkait

- [Memulai](./getting-started.md) — Panduan singkat task pertama Anda
- [Mengelola Task](./managing-tasks.md) — Mulai, hentikan, edit, hapus, dan cari
- [Auto-restart](./auto-restart.md) — Jaga task tetap berjalan setelah crash
- [Penjadwalan (Cron)](./scheduling.md) — Jalankan task sesuai jadwal waktu
- [Dependensi & Jeda Mulai](./dependencies.md) — Kendalikan urutan dan waktu mulai
- [Elevasi Admin](./admin-elevation.md) — Menjalankan task dengan hak istimewa yang ditinggikan

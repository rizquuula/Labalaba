# Memulai

Jalankan program pertama Anda dengan Labalaba dalam waktu kurang dari dua menit.

---

## Apa itu task?

Sebuah **task** adalah program yang ingin Anda kelola dengan Labalaba — server, skrip, background worker, atau apa pun yang dapat dijalankan di komputer Anda. Anda mendefinisikannya sekali (nama, path, argumen opsional), dan Labalaba akan mengingatnya sehingga Anda dapat memulai, menghentikan, memantau, dan me-restart-nya kapan saja.

---

## Task pertama Anda dalam tiga langkah

Contoh ini menjalankan server HTTP Python sederhana, tetapi langkah-langkahnya sama untuk program apa pun.

### Langkah 1 — Buka formulir task

Klik tombol **New Task** di jendela utama (atau tombol **Add your first task** jika daftar Anda masih kosong). Formulir **New Task** akan terbuka.

### Langkah 2 — Isi informasi dasar

| Kolom | Yang perlu dimasukkan (contoh) |
|---|---|
| **Description** | `My Python Server` |
| **Executable / Script Path** | `server.py` (atau klik **Browse** untuk memilih berkas) |
| **Python Runner** | `python` (kolom ini muncul otomatis untuk berkas `.py`) |
| **Arguments** | `--port 8080` |

> **Tip:** Klik **Browse** untuk menggunakan pemilih berkas alih-alih mengetikkan path secara manual. Labalaba mendeteksi runner yang tepat untuk skrip secara otomatis.

Klik **Create Task**. Formulir akan tertutup dan task baru Anda muncul di daftar dengan lencana **stopped**.

### Langkah 3 — Jalankan

Pada kartu task, klik **Start**. Lencana berubah menjadi hijau (**running**) dan Labalaba mulai melacak PID, CPU, serta penggunaan memorinya.

---

## Pantau log

Klik **View Logs** pada kartu task untuk membuka penampil log secara langsung. Semua yang ditulis program ke stdout atau stderr akan muncul di sini secara real time.

Tekan **Escape** untuk menutup penampil log.

---

## Hentikan task

Klik **Stop** pada kartu task. Dialog konfirmasi akan muncul — klik **Stop** sekali lagi untuk mengonfirmasi. Lencana kembali ke **stopped** (abu-abu).

> **Catatan:** Menghentikan task yang keluar dengan bersih (exit code 0) adalah penghentian yang normal dan disengaja — Labalaba tidak memperlakukannya sebagai crash.

---

## Selesai

Task Anda tersimpan dan akan selalu muncul di daftar. Anda dapat menjalankannya kembali kapan saja hanya dengan satu klik.

---

## Langkah Berikutnya

- [Membuat Task](./creating-tasks.md) — Penjelasan lengkap setiap kolom di formulir task
- [Mengelola Task](./managing-tasks.md) — Cari, filter, edit, hapus, dan pahami lencana status
- [Melihat Log](./logs.md) — Lebih lanjut tentang penampil log dan berkas log di disk
- [Auto-restart](./auto-restart.md) — Jalankan task secara otomatis jika mengalami crash

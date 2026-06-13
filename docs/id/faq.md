# FAQ

Jawaban atas pertanyaan yang paling sering diajukan tentang Labalaba.

---

**Apakah Labalaba gratis dan open source?**

Ya. Labalaba berlisensi MIT dan bebas digunakan, dimodifikasi, serta didistribusikan. Kode sumber tersedia di GitHub di `https://github.com/rizquuula/labalaba`.

---

**Platform apa saja yang didukung Labalaba?**

Windows (x64), Linux (x64, AppImage), dan macOS (Intel dan Apple Silicon).

---

**Apakah saya perlu menjalankan server atau daemon terpisah?**

Tidak. Mesin yang memantau dan mengendalikan task-task Anda tertanam langsung di dalam aplikasi desktop dan berjalan otomatis saat Anda membukanya. Tidak ada yang perlu diinstal atau dijalankan secara terpisah.

---

**Di mana task dan log saya disimpan?**

Secara default keduanya berada di direktori data aplikasi (folder yang sama dengan berkas binary Labalaba). Anda akan menemukan `tasks.yaml` (definisi task), `settings.yaml` (preferensi), dan folder `logs/` (berkas log per task). Lihat [Berkas Konfigurasi](./configuration-files.md) untuk detail selengkapnya.

---

**Bisakah saya memindahkan konfigurasi ke komputer lain?**

Bisa. Salin `tasks.yaml` dan `settings.yaml` ke lokasi relatif yang sama di mesin baru (atau ke mana pun direktori data Anda berada). Task dan pengaturan Anda akan dipulihkan saat Anda membuka Labalaba berikutnya.

---

**Jika saya menutup jendela, apakah task-task saya tetap berjalan?**

Menutup jendela Labalaba akan keluar dari aplikasi, yang sekaligus menghentikan mesin internal. Task-task Anda tidak lagi dipantau. Saat Anda membuka Labalaba kembali, aplikasi akan memeriksa proses yang sebelumnya berjalan masih hidup atau tidak berdasarkan PID-nya, lalu menandainya sebagai **running** atau **crashed** — namun tidak dapat mengambil kembali output yang mereka hasilkan saat aplikasi tertutup.

---

**Apa perbedaan antara "stopped" dan "crashed"?**

- **Stopped** — task tidak berjalan karena Anda menghentikannya secara manual, atau belum pernah dimulai.
- **Crashed** — task keluar sendiri dengan error (kode keluar non-zero). Program yang keluar dengan bersih menggunakan kode 0 ditampilkan sebagai **stopped**, bukan crashed.

---

**Mengapa log kosong untuk task "Run as Admin"?**

Di Windows, proses yang dielevasi via UAC berjalan dalam konteks keamanan terpisah yang mencegah Labalaba menangkap outputnya. Ini adalah keterbatasan Windows — tidak ada yang dapat dilakukan dari dalam aplikasi. Di macOS dan Linux, toggle **Run as Admin** tidak berpengaruh; gunakan `sudo` dalam argumen perintah Anda. Lihat [Admin Elevation](./admin-elevation.md).

---

**Bisakah saya menjalankan skrip Python tanpa mengubah PATH saya?**

Bisa. Saat Anda mengatur **Executable / Script Path** task ke berkas yang berekstensi `.py` atau `.pyw`, dropdown **Python Runner** akan muncul. Pilih dari `python`, `pythonw`, `uv run`, `pipenv run python`, `poetry run python`, atau masukkan perintah kustom. Labalaba akan menggunakan interpreter tersebut untuk menjalankan skrip Anda sehingga Anda tidak perlu menambahkan apa pun ke PATH sistem.

---

**Bisakah saya menjalankan task berdasarkan jadwal?**

Bisa. Atur field **Cron Schedule** saat membuat atau mengedit task. Gunakan string cron 6-field (detik, menit, jam, hari-dalam-bulan, bulan, hari-dalam-minggu) dan perhatikan bahwa waktu menggunakan UTC. Lihat [Scheduling (Cron)](./scheduling.md) untuk contoh dan referensi field lengkap.

---

**Bagaimana cara menjalankan task secara otomatis saat komputer dinyalakan?**

Aktifkan **Launch at login** di Settings (Settings → bagian **Notifications** → **Launch at login**). Labalaba akan terbuka secara otomatis saat Anda masuk, dan task apa pun yang memiliki **Auto-restart** aktif akan dilanjutkan seperti biasa.

---

## Terkait

- [Getting Started](./getting-started.md)
- [Membuat Task](./creating-tasks.md)
- [Mengelola Task](./managing-tasks.md)
- [Auto-restart](./auto-restart.md)
- [Scheduling (Cron)](./scheduling.md)
- [Admin Elevation](./admin-elevation.md)
- [Berkas Konfigurasi](./configuration-files.md)
- [Pemecahan Masalah](./troubleshooting.md)
- [Kembali ke Beranda](./README.md)

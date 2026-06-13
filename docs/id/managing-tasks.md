# Mengelola Task

Mulai, hentikan, restart, cari, dan pantau task Anda — semua yang bisa dilakukan oleh daftar task dan kartu task.

---

## Bilah statistik

Bilah atas selalu menampilkan ringkasan langsung dari semua task Anda:

| Indikator | Warna | Arti |
|---|---|---|
| **Running** | Hijau | Task yang sedang aktif |
| **Stopped** | Abu-abu | Task yang tidak berjalan |
| **Crashed** | Merah | Task yang keluar dengan error |
| **Total** | — | Jumlah total task yang terdefinisi |

Jumlah diperbarui secara otomatis sekitar setiap 2 detik.

---

## Daftar task

Di bawah header **TASKS** Anda akan menemukan setiap task yang telah dibuat. Daftar ini diperbarui secara otomatis sekitar setiap 2 detik, sehingga perubahan status muncul dengan cepat tanpa perlu muat ulang secara manual.

### Pencarian dan pemfilteran

Setelah Anda memiliki setidaknya satu task, dua kontrol muncul di atas daftar:

- **Search tasks…** — ketik bagian mana pun dari nama task untuk mempersempit daftar secara instan.
- Dropdown filter status — pilih **All statuses**, **Running**, **Stopped**, atau **Crashed** untuk menampilkan hanya task dalam status tersebut.

Jika tidak ada task yang cocok dengan pencarian Anda, daftar akan menampilkan: "No tasks match your search."

---

## Anatomi kartu task

Setiap task ditampilkan sebagai sebuah kartu. Berikut arti setiap bagiannya:

```
┌─────────────────────────────────────────────────────────┐
│  [STATUS BADGE]  Task Name                              │
│  /path/to/executable   PID 12345   CPU: 2.1%   Mem: 45MB│
│  [ADMIN]  [AUTO-RESTART]                                │
│  [ Stop ] [ Restart ] [ View Logs ] [ Edit ] [ Delete ] │
└─────────────────────────────────────────────────────────┘
```

| Elemen | Detail |
|---|---|
| **Status badge** (lencana status) | Pill berwarna yang menunjukkan status saat ini (lihat tabel di bawah) |
| Nama task | **Description** yang Anda berikan pada task |
| Path executable | Ditampilkan dalam monospace; terlihat saat tersedia |
| **PID** | ID proses yang sedang berjalan (hanya task yang running) |
| **CPU** | Penggunaan CPU saat ini dalam persentase (hanya task yang running) |
| **Memory** | Penggunaan memori residen dalam MB (hanya task yang running) |
| Tag **ADMIN** | Ditampilkan jika task dikonfigurasi untuk berjalan sebagai Administrator |
| Tag **AUTO-RESTART** | Ditampilkan jika auto-restart saat crash diaktifkan |
| Border kartu | Berwarna **hijau** saat running, **merah** saat crashed |

Angka CPU dan memori diperbarui sekitar setiap 5 detik selama task berjalan.

---

## Arti lencana status

| Status | Warna | Artinya |
|---|---|---|
| **stopped** | Abu-abu | Task tidak berjalan |
| **starting** | Kuning | Proses peluncuran sedang berlangsung (termasuk jeda mulai yang dikonfigurasi) |
| **running** | Hijau | Task aktif dan sedang dipantau |
| **stopping** | — | Permintaan stop telah dikirim; proses sedang menutup diri |
| **crashed** | Merah | Task keluar dengan kode error non-zero dan tidak di-auto-restart |

> **Catatan:** Task yang keluar dengan **kode 0** dianggap sebagai penghentian normal yang disengaja — bukan crash. Status akan ditampilkan sebagai **stopped** (abu-abu), meskipun auto-restart diaktifkan.

---

## Memulai task

1. Temukan kartu task (gunakan kotak pencarian atau scroll).
2. Klik **Start**.

Lencana berpindah dari **stopped** → **starting** → **running** saat proses dijalankan.

---

## Menghentikan task

1. Klik **Stop** pada kartu task yang sedang berjalan.
2. Dialog konfirmasi bertajuk **Stop Task** akan muncul.
3. Konfirmasi untuk menghentikan proses.

Lencana berpindah ke **stopping**, kemudian **stopped** setelah proses selesai.

---

## Me-restart task

1. Klik **Restart** pada kartu task yang sedang berjalan.
2. Dialog konfirmasi muncul: "stop and start it again".
3. Konfirmasi untuk restart.

Labalaba menghentikan proses dan segera menjalankannya kembali.

> **Tip:** Gunakan **Restart** setelah mengubah berkas konfigurasi yang dibaca proses saat startup — tidak perlu menghentikan dan memulai secara manual.

---

## Melihat log

Klik **View Logs** pada kartu task mana pun untuk membuka penampil log secara langsung. Stdout dan stderr dari proses akan mengalir secara real time.

Tekan **Escape** untuk menutup penampil log. Lihat [Melihat Log](./logs.md) untuk informasi lebih lanjut, termasuk cara mengakses berkas log di disk.

---

## Mengedit task

1. Klik **Edit** pada kartu task mana pun.
2. Formulir **Edit Task** terbuka, sudah terisi dengan nilai saat ini.
3. Ubah kolom yang diinginkan dan klik **Save Changes**.

> **Tip:** Anda dapat mengedit task yang sedang berjalan. Perubahan akan berlaku saat task dijalankan kembali berikutnya.

---

## Menghapus task

1. Klik **Delete** (tombol merah) pada kartu task.
2. Dialog konfirmasi berbahaya akan muncul:
   > "Delete task? This permanently removes '**Task Name**' and cannot be undone."
3. Konfirmasi untuk menghapus.

> **Peringatan:** Penghapusan bersifat permanen. Definisi task langsung dihapus dan tidak dapat dipulihkan. Berkas log di disk tidak dihapus secara otomatis.

---

## Error pada aksi

Jika aksi mulai, hentikan, restart, atau aksi lainnya gagal, pesan error berwarna merah akan muncul langsung di bawah baris meta task pada kartu tersebut.

---

## Status koneksi

| Pesan | Arti |
|---|---|
| "Connecting to daemon…" | Aplikasi sedang memulai; mesin sedang diinisialisasi |
| "Connection lost — showing last known state" | Koneksi langsung ke mesin latar belakang terputus; data yang ditampilkan mungkin tidak terbaru |
| "Cannot connect to daemon" | Mesin latar belakang tidak dapat dijangkau |

Jika Anda melihat error koneksi yang terus-menerus, lihat [Pemecahan Masalah](./troubleshooting.md).

---

## Terkait

- [Membuat Task](./creating-tasks.md) — Referensi lengkap untuk setiap kolom formulir task
- [Melihat Log](./logs.md) — Penampil log dan berkas log di disk
- [Auto-restart](./auto-restart.md) — Pulihkan task yang crash secara otomatis
- [Penjadwalan (Cron)](./scheduling.md) — Jalankan task sesuai jadwal waktu
- [Pemecahan Masalah](./troubleshooting.md) — Bantuan untuk error koneksi dan masalah lainnya

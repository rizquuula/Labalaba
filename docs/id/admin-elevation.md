# Elevasi Admin

Beberapa program memerlukan hak akses yang ditingkatkan untuk berjalan — misalnya, untuk mengikat ke port bernomor rendah, mengakses sumber daya sistem yang dilindungi, atau mengubah konfigurasi sistem. Labalaba menyediakan tombol **Run as Admin** (jalankan sebagai admin) untuk menangani hal ini di Windows.

## Cara mengaktifkannya

1. Buka formulir task (buat task baru atau edit task yang sudah ada).
2. Pindah ke tab **Advanced**.
3. Aktifkan tombol **Run as Admin**.
4. Simpan task.

Label **ADMIN** akan muncul pada kartu task ketika opsi ini diaktifkan.

## Perilaku per platform

### Windows

Saat **Run as Admin** aktif, Labalaba meluncurkan program melalui prompt UAC (User Account Control). Windows akan meminta Anda mengonfirmasi elevasi sebelum proses dimulai.

> **Peringatan:** Karena proses yang dielevasi berjalan dalam sesi Windows yang terpisah, Labalaba **tidak dapat menangkap stdout atau stderr-nya**. Artinya, penampil log langsung akan kosong atau hanya menampilkan output yang sangat terbatas untuk task yang dielevasi. Jika Anda perlu melihat output dari program yang dielevasi, arahkan outputnya ke sebuah file di dalam program itu sendiri.

### macOS dan Linux

Tombol **Run as Admin** **tidak berpengaruh** pada macOS atau Linux. Jika task Anda memerlukan hak akses yang ditingkatkan di platform ini, gunakan `sudo` di dalam perintah atau runner Anda secara langsung.

Misalnya, alih-alih mengandalkan tombol tersebut, atur berkas executable atau perintah Anda untuk menyertakan `sudo`:

```
sudo /path/to/your/program --your-flags
```

> **Catatan:** Menggunakan `sudo` dalam sebuah perintah mungkin tetap meminta kata sandi di terminal tempat Labalaba diluncurkan, tergantung pada konfigurasi `sudo` sistem Anda.

## Kapan menggunakan elevasi admin

Gunakan **Run as Admin** saat program Anda benar-benar membutuhkannya — misalnya:

- Mengikat ke port di bawah 1024 pada Windows.
- Menginstal atau mengelola layanan Windows.
- Mengakses direktori sistem yang dilindungi.

## Peringatan keamanan

Menjalankan program dengan hak akses yang ditingkatkan mengandung risiko. Bug atau muatan berbahaya dalam proses yang dielevasi dapat memengaruhi seluruh sistem Anda. Aktifkan **Run as Admin** hanya untuk program yang Anda percaya dan yang benar-benar memerlukannya. Utamakan menjalankan task tanpa elevasi kapan pun memungkinkan.

## Terkait

- [Membuat Task](./creating-tasks.md)
- [Log](./logs.md)
- [Pemecahan Masalah](./troubleshooting.md)
- [FAQ](./faq.md)
- [Kembali ke Beranda](./README.md)

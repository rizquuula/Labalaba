# Dependensi

Saat menjalankan sekumpulan task secara bersamaan, Anda sering kali perlu task-task tersebut dimulai dalam urutan tertentu — misalnya, database sebelum server API. Labalaba menyediakan dua cara untuk mengatur hal ini: **Startup Delay** dan `depends_on`.

## Startup Delay (jeda mulai)

**Startup Delay (ms)** tersedia di tab **Advanced** pada formulir task.

Saat Anda memulai task yang sudah dikonfigurasi dengan jeda, Labalaba akan menunggu sejumlah milidetik tersebut sebelum benar-benar meluncurkan proses. Selama menunggu, task menampilkan status **starting**. Setelah jeda berlalu, proses dimulai seperti biasa.

| Nilai delay | Waktu tunggu sebenarnya |
|-------------|-------------------------|
| `0`         | Tanpa jeda (default)    |
| `1000`      | 1 detik                 |
| `5000`      | 5 detik                 |
| `10000`     | 10 detik                |

> **Tip:** Gunakan teks petunjuk sebagai pengingat — kolom menampilkan "In milliseconds (5000 = 5 seconds)".

Startup delay adalah cara yang sederhana dan andal untuk memberi waktu pada sebuah layanan agar siap sebelum layanan berikutnya mencoba terhubung ke dalamnya.

## Dependensi (`depends_on`)

Pengaturan `depends_on` memberi tahu Labalaba task mana saja yang harus sudah berjalan sebelum task ini dimulai. Pengaturan ini bekerja berdampingan dengan startup delay untuk memberikan urutan yang presisi.

> **Catatan:** Tidak ada kolom **Dependencies** di formulir task. Anda mengatur `depends_on` dengan mengedit file konfigurasi `tasks.yaml` secara langsung.

Tambahkan daftar `depends_on` pada sebuah task di `tasks.yaml`, dengan merujuk pada nilai **id** dari task-task yang harus dimulai terlebih dahulu:

```yaml
tasks:
  - id: "db-task-id"
    description: "Database"
    # ... kolom lainnya ...

  - id: "api-task-id"
    description: "API Server"
    startup_delay_ms: 5000
    depends_on: ["db-task-id"]
    # ... kolom lainnya ...
```

Pada contoh di atas, task **API Server** tidak akan dimulai sampai **Database** berjalan, dan ia juga akan menunggu 5 detik tambahan (melalui `startup_delay_ms`) sebelum diluncurkan, memberi waktu bagi database untuk menyelesaikan inisialisasinya.

## Mengurutkan sekumpulan task kecil

Berikut adalah pola untuk menjalankan tiga task secara berurutan:

```yaml
tasks:
  - id: "database"
    description: "Database"
    # Tanpa jeda — langsung dimulai

  - id: "cache"
    description: "Cache"
    startup_delay_ms: 3000
    depends_on: ["database"]
    # Menunggu database, lalu menunggu 3 detik lagi

  - id: "api"
    description: "API Server"
    startup_delay_ms: 5000
    depends_on: ["database", "cache"]
    # Menunggu database dan cache, lalu menunggu 5 detik lagi
```

> **Peringatan:** Mengedit `tasks.yaml` secara langsung memerlukan kehati-hatian. Buat cadangan sebelum mengedit, dan pastikan indentasi YAML Anda benar. Lihat [File Konfigurasi](./configuration-files.md) untuk panduan lebih lanjut.

## Terkait

- [Membuat Task](./creating-tasks.md)
- [Mengelola Task](./managing-tasks.md)
- [Penjadwalan](./scheduling.md)
- [File Konfigurasi](./configuration-files.md)
- [Kembali ke Beranda](./README.md)

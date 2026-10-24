# CheckMat

CheckMat berawal dari kebutuhan sederhana untuk mengelola tugas sehari-hari dengan lebih rapi dan praktis. Dikembangkan sebagai proyek personal, aplikasi ini dirancang untuk membantu pekerjaan terasa lebih teratur tanpa kerumitan yang tidak diperlukan.

Dengan mengutamakan kesederhanaan, kenyamanan, dan privasi, CheckMat terus dikembangkan sebagai alat bantu sehari-hari yang praktis, dengan harapan dapat memberi manfaat lebih luas.

## Fitur Utama

### Keamanan dan Privasi

- **Kunci dengan PIN:** Lindungi aplikasi menggunakan PIN 4–6 digit.
- **Kunci Otomatis:** Atur agar aplikasi terkunci otomatis setelah tidak digunakan selama waktu tertentu.
- **Data Pribadi:** Data tugas disimpan di perangkat dengan perlindungan enkripsi.
- **Satu Jendela Aplikasi:** Mencegah aplikasi terbuka berkali-kali secara tidak sengaja.

### Ruang Kerja dan Kategori

- **Banyak Ruang Kerja:** Pisahkan tugas berdasarkan proyek, pekerjaan, atau kebutuhan pribadi.
- **Kategori Berwarna:** Kelompokkan tugas berdasarkan kategori dan gunakan warna agar lebih mudah dikenali.
- **Atur Sesuai Keinginan:** Susun urutan ruang kerja dan kategori dengan mudah melalui fitur geser dan lepas (_drag and drop_).

### Pengelolaan Tugas

- **Status Tugas:** Pisahkan tugas menjadi tiga bagian, yaitu Todo, Selesai, dan Dihapus, agar lebih mudah dipantau.
- **Tag untuk Tugas:** Tambahkan tag seperti `#kerja`, `#pribadi`, atau `#penting` untuk memudahkan pencarian dan pengelompokan.
- **Cari dan Filter:** Temukan tugas dengan cepat melalui pencarian teks atau pilih tag tertentu.
- **Checklist Subtugas:** Pecah pekerjaan besar menjadi langkah-langkah kecil dan pantau progresnya.
- **Detail dan Riwayat Tugas:** Lihat informasi tugas serta riwayat waktu pembuatan, pengerjaan, dan penyelesaiannya.
- **Edit dengan Praktis:** Ubah isi tugas langsung dari daftar atau melalui halaman detail tanpa perlu repot.
- **Salin Teks dengan Sekali Klik:** Tandai teks penting menggunakan format khusus agar bisa disalin dengan cepat.
- **Bandingkan Tugas:** Bandingkan beberapa tugas untuk membantu meninjau informasi dan perbedaannya.

### Pengelolaan Banyak Tugas Sekaligus

Kelola beberapa tugas dalam satu waktu tanpa harus mengubahnya satu per satu.

- Tandai banyak tugas sebagai selesai.
- Ubah status beberapa tugas sekaligus.
- Pindahkan tugas ke kategori lain.
- Gandakan beberapa tugas.
- Hapus banyak tugas sekaligus.

### Tampilan dan Produktivitas

- **Mode Terang dan Gelap:** Pilih tampilan yang nyaman digunakan, baik siang maupun malam.
- **Pengurutan Tugas:** Susun daftar berdasarkan urutan pilihan, kategori, waktu pembaruan, atau nama tugas.

## Persiapan Sebelum Menjalankan

Untuk menjalankan CheckMat dalam mode pengembangan di Windows, siapkan:

- [Node.js](https://nodejs.org/) versi LTS beserta npm.
- [Rust](https://www.rust-lang.org/).
- MinGW-w64.
- WebView2 Runtime.

## Cara Menjalankan

Jika ingin menjalankan CheckMat dari kode sumber, buka terminal dan jalankan perintah berikut:

```bash
cd frontend-svelte
npm install
npm run build
cd ../src-tauri
cargo run
```

## Cara Membuat Aplikasi EXE

### Menggunakan Script

Untuk membuat file aplikasi yang dapat dijalankan di Windows, jalankan file berikut dari direktori utama proyek:

    build_exe.bat

### Menggunakan Build Manual

Kamu juga bisa membangunnya secara manual:

    cd frontend-svelte
    npm run build
    cd ../src-tauri
    cargo build --release

Setelah proses build berhasil, file aplikasi dapat ditemukan di:

    src-tauri\target\x86_64-pc-windows-gnu\release\checkmat.exe

## Struktur Folder

    taskmini-tauri/
    ├── frontend-svelte/     # Kode tampilan aplikasi
    ├── frontend/            # File tampilan siap digunakan
    ├── src-tauri/           # Bagian utama aplikasi
    ├── build_exe.bat        # Script untuk membuat file EXE
    └── README.md            # Dokumentasi aplikasi

---

**CheckMat — Catat, kelola, dan selesaikan satu per satu.**

# CheckMat

Aplikasi desktop to-do list sederhana yang dibuat berdasarkan kebutuhan pribadi untuk mencatat dan menceklis pekerjaan agar lebih mudah dikelola, praktis digunakan, dan sesuai kebutuhan. Dilengkapi penyimpanan data lokal terenkripsi serta fitur perlindungan menggunakan PIN.

## Fitur

- Kunci PIN (4-6 digit)
- Multi-project dan pengelolaan kategori tugas
- 3 tab status: Task, Selesai, Dihapus
- Drag & drop untuk mengubah urutan tugas dan kategori
- Berbagai opsi pengurutan tugas (posisi, kategori, waktu, nama)
- Single instance (hanya satu aplikasi yang berjalan bersamaan)
- Tema gelap dan terang
- Export dan import CSV

## Tech Stack

- Desktop: Tauri v2, Rust, rusqlite, sha2
- Frontend: Svelte 5, TailwindCSS v4, Vite, Bits UI / shadcn-svelte

## Prasyarat (Windows)

- Node.js LTS & npm
- Rust target GNU (`rustup default stable-x86_64-pc-windows-gnu`)
- MinGW-w64 gcc di PATH
- WebView2 Runtime
- Python 3 (untuk `gen_icons.py`)

## Cara Menjalankan (Dev)

```bash
py gen_icons.py
cd frontend-svelte && npm install && npm run build && cd ..
cd src-tauri && cargo run
```

## Cara Build EXE

Jalankan script otomatis:

```cmd
build_exe.bat
```

Atau secara manual:

```cmd
cd frontend-svelte && npm run build && cd ..
py gen_icons.py
cd src-tauri && cargo build --release
```

Hasil file:
`src-tauri\target\x86_64-pc-windows-gnu\release\checkmat.exe`

## Struktur Folder

```text
taskmini-tauri/
├── frontend-svelte/     # Source UI (Svelte 5, Tailwind v4, Vite)
├── frontend/            # Hasil build static frontend untuk Tauri
├── src-tauri/           # Backend Rust & konfigurasi Tauri
│   ├── src/main.rs      # Setup window & handler command
│   └── src/db.rs        # SQLite in-memory, enkripsi XOR, data & PIN
├── build_exe.bat        # Script build 1-klik (frontend + EXE)
├── gen_icons.py         # Generator icon
└── README.md
```

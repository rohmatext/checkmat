# Tutorial Build TaskMini (Tauri + Svelte 5)

Panduan lengkap dari nol sampai EXE jadi. Semua perintah dijalankan dari
**Command Prompt / PowerShell biasa**, bukan dari terminal agent yang bisa nyangkut.

---

## 0. Prasyarat (sekali saja per mesin)

| Kebutuhan            | Cek dengan              | Kalau belum ada                                                       |
| -------------------- | ----------------------- | --------------------------------------------------------------------- |
| **Node.js + npm**    | `node -v` dan `npm -v`  | install Node LTS (nvm4w sudah dipakai di mesin ini)                   |
| **Rust stable**      | `rustc -V`              | `rustup default stable-x86_64-pc-windows-gnu`                         |
| **MinGW-w64 gcc**    | `gcc --version`         | install WinLibs POSIX/UCRT, tambahkan `<winlibs>\mingw64\bin` ke PATH |
| **windres**          | `windres --version`     | ikut paket MinGW di atas                                              |
| **WebView2 Runtime** | Windows 11 sudah bawaan | install "Evergreen Runtime" dari Microsoft                            |
| **Python**           | `py -V`                 | dipakai `gen_icons.py` untuk bikin ikon                               |

Kenapa target GNU? Mesin ini **tidak punya MSVC C++ build tools**. File
`src-tauri/.cargo/config.toml` memaksa target `x86_64-pc-windows-gnu` + linker `gcc`:

```toml
[build]
target = "x86_64-pc-windows-gnu"

[target.x86_64-pc-windows-gnu]
linker = "gcc"
ar = "ar"
```

---

## 1. Build frontend (Svelte 5 → HTML statis)

Sumber UI ada di `taskmini-tauri/frontend-svelte/`. Hasil build **keluar** ke
`taskmini-tauri/frontend/` (lihat `vite.config.js`, adapter-static).

```
cd taskmini-tauri\frontend-svelte
npm install          :: hanya pertama kali atau kalau package.json berubah
npm run build
```

Sukses ditandai:

```
> Using @sveltejs/adapter-static
Wrote site to "../frontend"
✔ done
```

Verifikasi cepat — file ini harus ada dan berisi referensi `/_app/...`:

```
type taskmini-tauri\frontend\index.html
dir taskmini-tauri\frontend\_app\immutable\entry
```

> **Wajib urut:** frontend dulu, baru Rust. `cargo build` menyalin isi
> `frontend/` ke dalam EXE. Kalau urutannya kebalik, EXE berisi UI lama.

---

## 2. Generate ikon

```
cd taskmini-tauri
py gen_icons.py
```

Menghasilkan `src-tauri/icons/icon.ico` (dan PNG) dari skrip. Lewati kalau ikon
tidak berubah.

---

## 3. Build EXE Rust

**Penting — jalankan dari dalam folder `src-tauri`.** Kalau dijalankan dari
folder induk dengan `--manifest-path`, `.cargo/config.toml` tidak terbaca,
target jadi MSVC, dan build gagal / EXE masuk ke folder berbeda.

```
cd taskmini-tauri\src-tauri
cargo build --release
```

Build pertama 3–10 menit. Berikutnya jauh lebih cepat (incremental).

Tanda sukses:

```
Compiling taskmini v0.1.0
warning: `taskmini` (bin "taskmini") generated 1 warning
    Finished `release` profile [optimized] target(s) in 1m 36s
```

Warning berikut **aman, bisa diabaikan**:

```
linker stderr: ...ld.exe: .rsrc merge failure: multiple non-default manifests
```

Itu keluhan kosmetik `ld` (MinGW) soal resource manifest, bukan error.

---

## 4. Hasil + cara pakai

```
taskmini-tauri\src-tauri\target\x86_64-pc-windows-gnu\release\taskmini.exe
```

⚠️ **EXE butuh `WebView2Loader.dll` di folder yang sama.** Cargo sudah
menaruhnya otomatis di `target\...\release\`. Jadi kalau mau pindah atau
kirim ke orang lain, **bawa SATU FOLDER**, bukan hanya file `.exe`-nya.

Cara paling rapuh-tapi-mudah: klik kanan folder `release` → _Copy_, tempel
di lokasi lain. Isinya ± 5 file penting: `taskmini.exe`,
`WebView2Loader.dll`, plus beberapa `*.dll`/`*.pdb` pendamping.

**Data tugas disimpan di `data.bin` (terenkripsi) dan konfigurasi di `data.cfg`, SEBELAH file EXE** (mode portable — lihat
`db_dir()` di `db.rs`). Artinya: pindah folder = data ikut pindah. Kalau hanya
mengganti `checkmat.exe` di folder yang sama, data lama tetap terbaca.

---

## 5. Satu perintah (skrip otomatis)

File `taskmini-tauri\build_exe.bat` sudah disediakan:

```bat
@echo off
cd /d %~dp0
py gen_icons.py
cd src-tauri
cargo build --release
echo.
echo Selesai. Exe ada di: src-tauri\target\x86_64-pc-windows-gnu\release\taskmini.exe
pause
```

Kekurangan: **tidak** membuild frontend. Kalau kamu mengubah UI Svelte, harus
jalankan langkah 1 dulu. Versi lengkapnya:

```bat
@echo off
cd /d %~dp0

echo [1/3] Build frontend Svelte...
pushd frontend-svelte
call npm run build
if errorlevel 1 ( echo GAGAL di frontend & popd & pause & exit /b 1 )
popd

echo [2/3] Generate ikon...
py gen_icons.py

echo [3/3] Build EXE...
cd src-tauri
cargo build --release
if errorlevel 1 ( echo GAGAL di cargo & pause & exit /b 1 )

echo.
echo SUKSES: %~dp0src-tauri\target\x86_64-pc-windows-gnu\release\taskmini.exe
pause
```

---

## 6. Kalau macet

| Gejala                                             | Penyebab                               | Solusi                                          |
| -------------------------------------------------- | -------------------------------------- | ----------------------------------------------- |
| `error: linker `link.exe` not found`               | jalan dari folder induk → target MSVC  | `cd src-tauri` dulu                             |
| `config_file_unsupported` (SvelteKit)              | ada `svelte.config.js` di SvelteKit v3 | hapus file itu, pindah opsi ke `vite.config.js` |
| `module_removed_lib`                               | import `$lib/...`                      | ganti ke `#lib/...` (Kit v3)                    |
| EXE jalan tapi UI kosong/lama                      | lupa `npm run build`                   | ulangi langkah 1 lalu 3                         |
| `WebView2Loader.dll was not found` saat EXE dibuka | EXE dipindah sendirian                 | bawa seluruh isi folder `release`               |
| `a11y_...` saat build frontend                     | elemen drag tanpa role                 | tambah `role="listitem"` / `role="list"`        |

---

## 7. Ringkasan 3 perintah

```bat
cd taskmini-tauri\frontend-svelte && npm run build
cd ..\..\taskmini-tauri && py gen_icons.py
cd src-tauri && cargo build --release
```

Output:
`taskmini-tauri\src-tauri\target\x86_64-pc-windows-gnu\release\`

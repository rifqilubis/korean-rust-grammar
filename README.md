# korean-rust-grammar

Aplikasi terminal untuk belajar **tata bahasa Korea (한국어 문법)**, ditulis 100% dengan **Rust murni** — tanpa dependensi eksternal sama sekali, sehingga bisa di-build di mesin mana pun yang punya `cargo`, termasuk tanpa akses internet.

```
╭─ 한국어 문법 · Korean Grammar ─────────────────────────────────────────────╮
│ 12 topik · 39 soal · progres belum dimulai                                 │
│                                                                            │
│ Tingkat  No  Topik                                                         │
│ [1]  01  한글 — Huruf Korea: dasar membaca    belum dikerjakan             │
│ [1]  02  어순 — Urutan kalimat: SOV           belum dikerjakan             │
│ [1]  03  은/는 — Partikel topik kalimat       belum dikerjakan             │
│ ...                                                                        │
╰────────────────────────────────────────────────────────────────────────────╯
```

## Komposisi bahasa

| Bahasa | Baris kode | Porsi |
|---|---:|---:|
| **Rust** (`.rs`) | ~5.600 | **±92%** |
| Konfigurasi, dokumentasi & skrip build (TOML, Markdown, PowerShell) | ~460 | ±8% |

Tidak ada JavaScript, Python, Java, atau Kotlin di folder ini — seluruh logika
aplikasi, data materi, penyimpanan progres, penulisan tabel terminal, sampai
antarmuka APK (digambar `egui`) dikerjakan Rust. Satu-satunya berkas non-Rust di
dalam APK adalah `AndroidManifest.xml` yang dihasilkan `cargo-apk`.

## Fitur

- **12 topik materi** dari 한글 sampai penghubung kalimat, lengkap dengan tabel aturan, contoh, catatan, dan 39 soal kuis.
- **Mesin kuis interaktif** dengan pengacakan soal & opsi, penilaian langsung, serta rekap kesalahan di akhir sesi.
- **APK Android** dengan antarmuka sentuh — tanpa Gradle, tanpa Java/Kotlin, tanpa WebView (lihat bagian [Android](#android)).
- **Penyimpanan progres** dalam JSON di direktori data pengguna (otomatis mundur ke `.kg-data` bila folder standar tidak bisa ditulisi).
- **Modul 한글 sungguhan**: dekomposisi 초성/중성/종성, komposisi balik, romanisasi dengan aturan 연음 (resyllabification) dan 비음화 (asimilasi hidung).
- **Tabel sadar lebar CJK** — suku kata 한글 dihitung 2 kolom sehingga tabel tetap rapi.
- **Inti tanpa dependensi**: parser JSON, PRNG xorshift64\*, dan pewarnaan ANSI ditulis sendiri di atas `std`. Dependensi UI hanya ada di crate `android/`.

## Instalasi

```bash
cd korean-rust-grammar
cargo build --release
```

Biner ada di `target/release/kg` (`kg.exe` di Windows). Pasang ke `PATH` dengan:

```bash
cargo install --path .
```

## Cara pakai

```bash
kg                        # dasbor: semua topik + progres
kg list                   # daftar topik
kg list --level 2         # hanya tingkat menengah
kg show 3                 # baca materi 은/는
kg quiz 3                 # kuis topik 3
kg quiz eun-neun -n 5     # kuis 5 soal saja
kg search hujan           # cari kata kunci di seluruh materi
kg rom 안녕하세요          # romanisasi → annyeonghaseyo
kg hangul 한국어           # urai per 초성/중성/종성
kg random                 # satu contoh kalimat acak
kg progress               # lihat progres
kg reset --yes            # hapus progres
```

Saat kuis, jawab dengan `1`–`4` atau `a`–`d`; `h` menampilkan petunjuk, `q` keluar.

### Variabel lingkungan

| Variabel | Fungsi |
|---|---|
| `NO_COLOR=1` | Matikan warna ANSI |
| `KG_COLUMNS=100` | Paksa lebar tampilan (berguna saat output di-pipe) |
| `KG_DATA_DIR=./data` | Ubah lokasi berkas `progress.json` |

## Struktur proyek

```
korean-rust-grammar/
├── Cargo.toml
├── README.md
├── .gitignore
├── src/
│   ├── main.rs       # biner `kg` (hanya I/O standar)
│   ├── lib.rs        # dokumentasi + deklarasi modul
│   ├── cli.rs        # parser argumen, semua perintah, tampilan
│   ├── data.rs       # bank materi 12 topik + 39 soal (static)
│   ├── model.rs      # tipe Topic/Block/Rule/Example/QuizItem
│   ├── hangul.rs     # dekomposisi, komposisi, romanisasi 한글
│   ├── quiz.rs       # sesi kuis, penilaian, rekap kesalahan
│   ├── progress.rs   # simpan/muat progres JSON + fallback folder
│   ├── json.rs       # parser & penulis JSON mini
│   ├── rng.rs        # xorshift64* untuk pengacakan
│   ├── ui.rs         # warna ANSI, lebar CJK, panel, tabel, progress bar
│   └── bin/
│       └── xtask.rs  # task runner: build Android CLI semua ABI
├── android/          # crate APK: antarmuka egui (Rust murni)
│   ├── Cargo.toml    # metadata APK: paket, ABI, minSdk, ikon
│   ├── src/
│   │   ├── lib.rs    # titik masuk `android_main` + `run`
│   │   ├── app.rs    # layar beranda/materi/kuis/hasil/progres
│   │   ├── font.rs   # menanam NanumGothic untuk glif 한글
│   │   ├── theme.rs  # palet terang/gelap + gaya widget sentuh
│   │   └── bin/
│   │       └── desktop.rs   # uji UI yang sama di desktop
│   └── res/          # ikon launcher per densitas
├── assets/fonts/     # NanumGothic-Regular.ttf + OFL.txt
├── tools/
│   └── build-apk.ps1 # cari SDK/NDK/JDK, tulis config, build + tanda tangan
└── tests/
    └── cli_integration.rs   # uji end-to-end biner `kg`
```

## Android

Ada dua jalur Android di repo ini:

1. **APK sungguhan** dengan antarmuka sentuh (crate `android/`, lihat di bawah).
2. **Biner CLI** `kg` yang dijalankan di **Termux** atau lewat `adb shell`.

### 1. APK (antarmuka sentuh, Rust murni)

Crate `android/` membangun APK tanpa Gradle, tanpa Java/Kotlin, dan tanpa
WebView/HTML/CSS/JS. Antarmuka digambar `egui` di atas OpenGL lewat `eframe`;
`Activity` yang dipakai adalah `android.app.NativeActivity` bawaan Android yang
memuat `libkorean_rust_grammar_android.so`.

```powershell
# Windows (PowerShell) — cari SDK/NDK/JDK sendiri, lalu build + tanda tangan
powershell -ExecutionPolicy Bypass -File tools/build-apk.ps1

# hanya satu ABI (jauh lebih cepat saat mengembangkan)
powershell -ExecutionPolicy Bypass -File tools/build-apk.ps1 -Target aarch64-linux-android

# build debug (tanpa optimasi, APK besar, memakai kunci debug NDK)
powershell -ExecutionPolicy Bypass -File tools/build-apk.ps1 -Debug
```

Hasil: `android/target/release/apk/korean-rust-grammar.apk` (±8,5 MB, tiga ABI:
arm64-v8a, armeabi-v7a, x86_64). Paketnya `com.rifqilubis.koreanrustgrammar`,
minSdk 24, targetSdk 34.

Prasyarat: Rust + `cargo-apk`, Android SDK (build-tools + platform 34), NDK r27,
dan JDK 17. Skrip membuat `android/.cargo/config.toml` sendiri dari lokasi NDK,
jadi berkas itu tidak perlu di-commit.

Isi APK: satu-satunya kode non-Rust adalah `AndroidManifest.xml` yang
dihasilkan `cargo-apk`. Tidak ada WebView, tidak ada berkas aset terpisah —
materi dan huruf Korea (NanumGothic, OFL) ditanam ke dalam biner lewat
`include_bytes!`.

Layarnya: **beranda** (12 topik + progres), **materi** (tabel aturan, contoh,
catatan), **kuis** (soal, penilaian langsung, pembahasan), **hasil** (skor,
daftar kesalahan, ulangi), dan **progres**. Progres disimpan di folder data
aplikasi Android (`internal_data_path`), yang diteruskan ke pustaka inti lewat
`KG_DATA_DIR`.

### 2. Biner CLI di Termux / adb

Biner `kg` juga bisa di-cross-compile ke Android dan dijalankan sebagai CLI:

```bash
# 1. Pasang target Rust (sekali saja)
rustup target add aarch64-linux-android armv7-linux-androideabi \
                   x86_64-linux-android i686-linux-android

# 2. Set lokasi NDK (kalau tidak terdeteksi otomatis)
export ANDROID_NDK_HOME="$HOME/Android/Sdk/ndk/27.0.12077973"   # Linux/macOS
# $env:ANDROID_NDK_HOME = "$env:LOCALAPPDATA\Android\Sdk\ndk\27.0.12077973"  # Windows

# 3. Build
cargo run --bin xtask -- android              # semua ABI
cargo run --bin xtask -- android --abi arm64  # hanya arm64-v8a
```

Hasil build per ABI (ukuran release, NDK r27, API 21):

| ABI | Target triple | Ukuran |
|---|---|---:|
| arm64-v8a | `aarch64-linux-android` | ~505 KB |
| armeabi-v7a | `armv7-linux-androideabi` | ~376 KB |
| x86_64 | `x86_64-linux-android` | ~554 KB |
| x86 | `i686-linux-android` | ~547 KB |

Biner ditautkan ke `bionic` (`/system/bin/linker64`), bukan GLIBC, sehingga
tidak butuh `proot` untuk urusan pustaka.

### Menjalankan di perangkat

```bash
# Termux (paling praktis, tanpa root)
adb push target/aarch64-linux-android/release/kg /sdcard/kg
# lalu di Termux:
cp /sdcard/kg ~/ && chmod +x ~/kg && ~/kg

# atau langsung lewat adb (cocok untuk uji cepat)
adb push target/aarch64-linux-android/release/kg /data/local/tmp/kg
adb shell chmod +x /data/local/tmp/kg
adb shell /data/local/tmp/kg list
```

Karena aplikasi ini CLI, tampilan tabel dan kuis bekerja di terminal Termux
(lebar layar ponsel ±40–60 kolom; gunakan `KG_COLUMNS=48` bila tabel terpotong).
Progres disimpan di `$HOME/.kg-korean/` pada Termux, atau di `$TMPDIR` /
`/data/local/tmp` saat dijalankan lewat `adb shell`.

### Kalau ingin APK dengan WebView

APK di `android/` sudah berupa aplikasi asli tanpa WebView. Bila nanti ingin
antarmuka berbasis HTML (mis. berbagi kode dengan versi web), jalur Tauri 2
tetap terbuka: bungkus `korean_rust_grammar` sebagai pustaka, sediakan perintah
`#[tauri::command]` (mis. `quiz_answer`, `list_topics`), lalu frontend HTML.
Seluruh logika inti (`data`, `quiz`, `hangul`, `progress`, `json`) sudah berupa
pustaka murni tanpa I/O terminal, jadi siap dipakai ulang frontend apa pun.

## Pengujian

```bash
cargo test                      # 90 tes inti: unit, integrasi biner, xtask, doc-test
cd android && cargo test        # 4 tes antarmuka APK tanpa jendela
cargo clippy --all-targets
cargo fmt --check
```

Uji di `android/` menjalankan bingkai `egui` sungguhan lewat
`Context::run_ui` tanpa membuka jendela: seluruh layar digambar (beranda, 12
layar materi, kuis, hasil, progres), alur kuis dijawab sampai selesai untuk
memastikan progres tercatat, dan umpan balik jawaban salah muncul.

Yang diuji antara lain: round-trip dekomposisi seluruh 11.172 suku kata 한글, aturan romanisasi (연음, pemecahan gugus konsonan, penghilangan ㅎ, 비음화), kesesuaian romanisasi otomatis dengan 48 contoh di bank materi, integritas bank soal (semua jawaban valid, tidak ada opsi kosong), pengacakan opsi yang tidak menghilangkan jawaban, penilaian kuis, penyimpanan progres, parser JSON (termasuk escape `\uXXXX` dan surrogate pair), perhitungan lebar CJK, serta setiap perintah CLI lewat stdin/stdout tiruan dan proses biner sungguhan.

## Materi yang tersedia

| No | Topik | Tingkat |
|---:|---|---|
| 01 | 한글 — huruf Korea | Dasar |
| 02 | 어순 — urutan SOV | Dasar |
| 03 | 은/는 — partikel topik | Dasar |
| 04 | 이/가 — partikel subjek | Dasar |
| 05 | 을/를 — partikel objek | Dasar |
| 06 | 이에요/예요 — kopula | Dasar |
| 07 | 있어요/없어요 — ada/tidak ada | Dasar |
| 08 | 아요/어요 — kata kerja sopan | Dasar |
| 09 | 숫자 — angka Sino & asli Korea | Menengah |
| 10 | 았어요/었어요 — masa lalu | Menengah |
| 11 | 안 / -지 않다 — negasi | Menengah |
| 12 | -고 / -서 — penghubung klausa | Lanjutan |

## Lisensi

MIT.

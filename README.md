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
| **Rust** (`.rs`) | ~4.600 | **±97%** |
| Konfigurasi & dokumentasi (TOML, Markdown, `.gitignore`) | ~200 | ±3% |

Tidak ada JavaScript, Python, atau skrip lain di folder ini — seluruh logika aplikasi, data materi, penyimpanan progres, sampai penulisan tabel terminal dikerjakan Rust.

## Fitur

- **12 topik materi** dari 한글 sampai penghubung kalimat, lengkap dengan tabel aturan, contoh, catatan, dan 39 soal kuis.
- **Mesin kuis interaktif** dengan pengacakan soal & opsi, penilaian langsung, serta rekap kesalahan di akhir sesi.
- **Penyimpanan progres** dalam JSON di direktori data pengguna (otomatis mundur ke `.kg-data` bila folder standar tidak bisa ditulisi).
- **Modul 한글 sungguhan**: dekomposisi 초성/중성/종성, komposisi balik, romanisasi dengan aturan 연음 (resyllabification) dan 비음화 (asimilasi hidung).
- **Tabel sadar lebar CJK** — suku kata 한글 dihitung 2 kolom sehingga tabel tetap rapi.
- **Tanpa dependensi**: parser JSON, PRNG xorshift64\*, dan pewarnaan ANSI ditulis sendiri di atas `std`.

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
│       └── xtask.rs  # task runner: build Android semua ABI
└── tests/
    └── cli_integration.rs   # uji end-to-end biner `kg`
```

## Android

Biner `kg` bisa di-cross-compile ke Android dan dijalankan sebagai CLI di
**Termux** atau lewat `adb shell`. Ini bukan APK — lihat catatan di bawah.

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

### Kalau ingin APK sungguhan

Biner ini adalah CLI, jadi belum punya antarmuka sentuh. Dua jalur:

1. **Tauri 2** — bungkus `korean_rust_grammar` sebagai pustaka, sediakan
   perintah `#[tauri::command]` (mis. `quiz_answer`, `list_topics`) dan
   frontend HTML. Cara ini dipakai proyek `korean-grammar` di repo sebelah.
2. **JNI** — kompilasi `libkorean_rust_grammar.so` per ABI, panggil dari
   `MainActivity` lewat `System.loadLibrary`.

Seluruh logika inti (`data`, `quiz`, `hangul`, `progress`, `json`) sudah
berupa pustaka murni tanpa I/O terminal, jadi siap dipakai ulang oleh
frontend apa pun.

## Pengujian

```bash
cargo test          # 90 tes: unit, integrasi biner, xtask, dan doc-test
cargo clippy --all-targets
cargo fmt --check
```

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

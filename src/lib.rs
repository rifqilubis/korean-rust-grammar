//! # korean-rust-grammar
//!
//! Pustaka + biner untuk belajar tata bahasa Korea (한국어 문법) dari terminal.
//!
//! Seluruh kode ditulis dengan Rust murni dan hanya memakai `std`:
//!
//! | Modul      | Isi                                                        |
//! |------------|------------------------------------------------------------|
//! | [`model`]  | Tipe data materi: topik, blok, contoh, soal kuis           |
//! | [`data`]   | Bank materi 12 topik (한글 → penghubung kalimat)            |
//! | [`hangul`] | Dekomposisi/komposisi suku kata 한글 + romanisasi           |
//! | [`quiz`]   | Mesin kuis: pengacakan opsi, penilaian, rekap kesalahan     |
//! | [`progress`]| Penyimpanan progres belajar (JSON) di direktori data user  |
//! | [`json`]   | Parser + penulis JSON mini (tanpa serde)                    |
//! | [`rng`]    | PRNG xorshift64 untuk mengacak soal                         |
//! | [`ui`]     | Warna ANSI, penghitung lebar tampilan (CJK), tabel, panel   |
//!
//! ## Contoh
//!
//! ```
//! use korean_rust_grammar::{hangul, data};
//!
//! // 로마자: dekomposisi + romanisasi satu suku kata
//! let syl = hangul::decompose('한').unwrap();
//! assert_eq!(syl.lead, 18); // ㅎ
//! assert_eq!(hangul::romanize("한글"), "hangeul");
//!
//! // Cari topik berdasarkan id atau nomor
//! assert!(data::find("eun-neun").is_some());
//! assert!(data::find("3").is_some());
//! ```

pub mod cli;
pub mod data;
pub mod hangul;
pub mod json;
pub mod model;
pub mod progress;
pub mod quiz;
pub mod rng;
pub mod ui;

/// Versi crate (dipakai oleh `kg --version`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

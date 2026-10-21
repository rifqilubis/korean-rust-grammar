//! Biner desktop untuk menguji antarmuka yang sama tanpa perangkat Android.
//!
//! Berguna saat mengembangkan tampilan: `cargo run --bin kg-desktop` di
//! folder `android/`. Biner ini tidak ikut ke dalam APK.

fn main() -> eframe::Result<()> {
    korean_rust_grammar_android::run(None)
}

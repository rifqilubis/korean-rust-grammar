//! Pemuatan huruf Korea.
//!
//! `egui` hanya membawa huruf Latin bawaan, sehingga suku kata 한글 akan
//! tampil sebagai kotak kosong tanpa huruf tambahan. NanumGothic (lisensi OFL)
//! ditanam ke dalam biner lewat `include_bytes!`, jadi tidak ada berkas aset
//! yang perlu dipasang di perangkat.
//!
//! Ukuran huruf ±2 MB; karena `opt-level = "s"` dan `strip`, APK akhirnya
//! tetap di bawah 15 MB untuk tiga ABI.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId};

/// Berkas huruf Korea yang ditanam saat kompilasi.
const NANUM_GOTHIC: &[u8] = include_bytes!("../../assets/fonts/NanumGothic-Regular.ttf");

const KOREAN_FONT: &str = "korean";

/// Daftarkan huruf Korea sebagai cadangan setelah huruf bawaan `egui`.
///
/// Urutan penting: huruf bawaan tetap utama agar teks Latin tidak berubah
/// bentuk, lalu NanumGothic dipakai untuk glif yang tidak ada di sana.
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        KOREAN_FONT.to_owned(),
        Arc::new(FontData::from_static(NANUM_GOTHIC)),
    );

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(KOREAN_FONT.to_owned());
    }

    ctx.set_fonts(fonts);
}

/// Huruf khusus untuk aksara 한글 berukuran besar (kartu topik).
pub fn large() -> FontId {
    FontId::proportional(30.0)
}

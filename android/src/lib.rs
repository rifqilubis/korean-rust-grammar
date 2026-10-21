//! APK Android untuk korean-rust-grammar.
//!
//! Antarmuka digambar langsung oleh `egui` di atas OpenGL lewat `eframe` —
//! tidak ada WebView, HTML, CSS, maupun JavaScript di dalam APK. Seluruh
//! materi, kuis, romanisasi, dan penyimpanan progres memakai pustaka
//! `korean_rust_grammar` yang sama dengan versi CLI.
//!
//! Berkas ini hanya berisi titik masuk; layar-layarnya ada di [`app`].

pub mod app;
pub mod font;
pub mod theme;

use eframe::egui;

/// Titik masuk untuk semua platform.
///
/// Di Android, `eframe` memanggil fungsi ini dari `android_main` dengan
/// `AndroidApp` milik Activity native. Di desktop dipanggil dari `main`.
pub fn run(android_app: Option<AndroidAppHandle>) -> eframe::Result<()> {
    #[cfg_attr(not(target_os = "android"), allow(unused_mut))]
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 880.0])
            .with_min_inner_size([320.0, 480.0])
            .with_title("Korean Rust Grammar"),
        ..Default::default()
    };

    #[cfg(target_os = "android")]
    {
        options.android_app = android_app;
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = android_app;
    }

    eframe::run_native(
        "Korean Rust Grammar",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}

/// Alias agar tanda tangan `run` sama di semua platform.
#[cfg(target_os = "android")]
pub type AndroidAppHandle = winit::platform::android::activity::AndroidApp;

#[cfg(not(target_os = "android"))]
pub type AndroidAppHandle = ();

// ---------------------------------------------------------------- Android
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("kg-rust"),
    );
    log::info!("korean-rust-grammar dimulai");

    // Di Android tidak ada `HOME`/`APPDATA` yang bisa ditulisi; folder data
    // aplikasi yang sebenarnya hanya bisa diketahui lewat Activity. Variabel
    // `KG_DATA_DIR` dipakai pustaka inti sebagai penimpa lokasi progres.
    if let Some(dir) = app.internal_data_path() {
        let dir = dir.join("kg-korean");
        log::info!("folder progres: {}", dir.display());
        std::env::set_var("KG_DATA_DIR", &dir);
    } else {
        log::warn!("internal_data_path tidak tersedia; progres memakai folder sementara");
    }

    if let Err(error) = run(Some(app)) {
        log::error!("gagal menjalankan antarmuka: {error}");
    }
}

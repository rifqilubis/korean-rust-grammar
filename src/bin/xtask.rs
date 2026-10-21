//! Task runner kecil untuk membangun biner `kg` bagi Android.
//!
//! Ditulis dengan Rust (bukan skrip shell) agar seluruh perkakas proyek tetap
//! satu bahasa dan bisa dipakai di Windows, Linux, maupun macOS.
//!
//! ```text
//! cargo run --bin xtask -- android              # semua ABI, profil release
//! cargo run --bin xtask -- android --abi arm64  # hanya arm64-v8a
//! cargo run --bin xtask -- android --debug      # profil debug
//! ```
//!
//! NDK dicari lewat `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT`, lalu ke lokasi
//! pemasangan Android Studio yang umum.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Tingkat API minimum. 21 = Android 5.0; Termux modern butuh 24+.
const MIN_API: &str = "21";

/// Satu ABI Android yang didukung Rust.
struct Abi {
    /// Target triple cargo.
    target: &'static str,
    /// Prefix clang di dalam NDK.
    clang: &'static str,
    /// Nama ABI Android (untuk keterangan).
    name: &'static str,
    /// Folder ABI di dalam APK.
    apk_dir: &'static str,
}

const ABIS: &[Abi] = &[
    Abi {
        target: "aarch64-linux-android",
        clang: "aarch64-linux-android",
        name: "arm64-v8a",
        apk_dir: "arm64-v8a",
    },
    Abi {
        target: "armv7-linux-androideabi",
        clang: "armv7a-linux-androideabi",
        name: "armeabi-v7a",
        apk_dir: "armeabi-v7a",
    },
    Abi {
        target: "x86_64-linux-android",
        clang: "x86_64-linux-android",
        name: "x86_64",
        apk_dir: "x86_64",
    },
    Abi {
        target: "i686-linux-android",
        clang: "i686-linux-android",
        name: "x86",
        apk_dir: "x86",
    },
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(task) = args.first() else {
        print_help();
        return ExitCode::SUCCESS;
    };

    match task.as_str() {
        "android" => run_android(&args[1..]),
        "check" => {
            // Ringkas: cek host tetap sehat sebelum mencoba cross-compile.
            let status = Command::new("cargo")
                .args(["test", "--offline", "--quiet"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("host: semua tes lulus");
                    ExitCode::SUCCESS
                }
                Ok(_) => ExitCode::from(1),
                Err(e) => {
                    eprintln!("gagal menjalankan cargo: {e}");
                    ExitCode::from(1)
                }
            }
        }
        "-h" | "--help" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("tugas tidak dikenal: {other}");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "xtask — perkakas build korean-rust-grammar\n\
         \n\
         PENGGUNAAN\n\
         \x20 cargo run --bin xtask -- android [opsi]\n\
         \x20 cargo run --bin xtask -- check\n\
         \n\
         OPSI 'android'\n\
         \x20 --abi <semua|arm64|armv7|x86_64|x86>   ABI yang dibangun (default: semua)\n\
         \x20 --debug                              Pakai profil debug\n\
         \n\
         LINGKUNGAN\n\
         \x20 ANDROID_NDK_HOME   Lokasi NDK (mis. .../Sdk/ndk/27.0.12077973)"
    );
}

fn run_android(args: &[String]) -> ExitCode {
    let mut abi_filter = "semua".to_string();
    let mut debug = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" => {
                debug = true;
                i += 1;
            }
            "--abi" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--abi butuh nilai");
                    return ExitCode::from(2);
                };
                abi_filter = value.clone();
                i += 2;
            }
            other => {
                eprintln!("opsi tidak dikenal: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(ndk) = find_ndk() else {
        eprintln!(
            "NDK Android tidak ditemukan.\n\
             Set ANDROID_NDK_HOME ke folder NDK, mis.:\n\
             \x20 $env:ANDROID_NDK_HOME = \"$env:LOCALAPPDATA\\Android\\Sdk\\ndk\\27.0.12077973\""
        );
        return ExitCode::from(1);
    };
    println!("NDK : {}", ndk.display());

    let toolchain_bin = ndk
        .join("toolchains")
        .join("llvm")
        .join("prebuilt")
        .join(host_tag())
        .join("bin");
    if !toolchain_bin.is_dir() {
        eprintln!("folder toolchain tidak ada: {}", toolchain_bin.display());
        return ExitCode::from(1);
    }

    let selected: Vec<&Abi> = match abi_filter.as_str() {
        "semua" | "all" => ABIS.iter().collect(),
        "arm64" | "aarch64" => ABIS
            .iter()
            .filter(|a| a.target.starts_with("aarch64"))
            .collect(),
        "armv7" | "arm" => ABIS
            .iter()
            .filter(|a| a.target.starts_with("armv7"))
            .collect(),
        "x86_64" | "x64" => ABIS
            .iter()
            .filter(|a| a.target.starts_with("x86_64"))
            .collect(),
        "x86" | "i686" => ABIS
            .iter()
            .filter(|a| a.target.starts_with("i686"))
            .collect(),
        other => {
            eprintln!("ABI tidak dikenal: {other}");
            return ExitCode::from(2);
        }
    };

    let mut built: Vec<(String, PathBuf, u64, &'static str)> = Vec::new();

    for abi in selected {
        println!("\n==> {} ({})", abi.name, abi.target);

        let clang = toolchain_bin.join(format!("{}{}-clang.cmd", abi.clang, MIN_API));
        // Di Linux/macOS berkas linker tidak berekstensi .cmd.
        let clang = if clang.is_file() {
            clang
        } else {
            toolchain_bin.join(format!("{}{}-clang", abi.clang, MIN_API))
        };
        if !clang.is_file() {
            eprintln!("  ! clang tidak ditemukan: {}", clang.display());
            continue;
        }

        let ar = toolchain_bin.join(if cfg!(windows) {
            "llvm-ar.exe"
        } else {
            "llvm-ar"
        });
        let env_target = abi.target.replace('-', "_").to_uppercase();
        let env_cc = abi.target.replace('-', "_");

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--offline")
            .arg("--target")
            .arg(abi.target)
            .arg("--bin")
            .arg("kg");
        if !debug {
            cmd.arg("--release");
        }
        cmd.env(format!("CARGO_TARGET_{env_target}_LINKER"), &clang)
            .env(format!("CC_{env_cc}"), &clang)
            .env(format!("AR_{env_cc}"), &ar)
            .env(format!("CARGO_TARGET_{env_target}_AR"), &ar);

        match cmd.status() {
            Ok(status) if status.success() => {}
            Ok(status) => {
                eprintln!("  ! build gagal (exit {status})");
                return ExitCode::from(1);
            }
            Err(e) => {
                eprintln!("  ! gagal menjalankan cargo: {e}");
                return ExitCode::from(1);
            }
        }

        let profile = if debug { "debug" } else { "release" };
        let artifact = PathBuf::from("target")
            .join(abi.target)
            .join(profile)
            .join("kg");
        let size = std::fs::metadata(&artifact).map(|m| m.len()).unwrap_or(0);
        built.push((abi.name.to_string(), artifact, size, abi.apk_dir));
    }

    if built.is_empty() {
        return ExitCode::from(1);
    }

    println!("\nHasil build:");
    for (name, path, size, apk_dir) in &built {
        println!(
            "  {:<12} {:>10} B  {}  (lib/{}/kg)",
            name,
            size,
            path.display(),
            apk_dir
        );
    }

    println!(
        "\nCara menjalankan di perangkat Android:\n\
         \x20 1. Termux (tanpa root):\n\
         \x20      adb push target/aarch64-linux-android/release/kg /sdcard/kg\n\
         \x20      pkg install proot  # opsional\n\
         \x20      cp /sdcard/kg ~/ && chmod +x ~/kg && ~/kg\n\
         \x20 2. Via adb shell (cocok untuk ABI yang sama dengan perangkat):\n\
         \x20      adb push target/aarch64-linux-android/release/kg /data/local/tmp/kg\n\
         \x20      adb shell chmod +x /data/local/tmp/kg\n\
         \x20      adb shell /data/local/tmp/kg list\n\
         \n\
         Catatan: ini biner CLI, bukan APK. Untuk pengalaman Android penuh\n\
         (layar sentuh), bungkus pustaka ini dengan frontend Tauri atau JNI."
    );

    ExitCode::SUCCESS
}

/// Folder prebuilt NDK untuk host saat ini.
fn host_tag() -> &'static str {
    if cfg!(windows) {
        "windows-x86_64"
    } else if cfg!(target_os = "macos") {
        "darwin-x86_64"
    } else {
        "linux-x86_64"
    }
}

/// Cari NDK dari variabel lingkungan atau lokasi pemasangan umum.
fn find_ndk() -> Option<PathBuf> {
    for key in ["ANDROID_NDK_HOME", "ANDROID_NDK_ROOT", "NDK_HOME"] {
        if let Ok(value) = std::env::var(key) {
            let path = PathBuf::from(value.trim());
            if is_ndk(&path) {
                return Some(path);
            }
        }
    }

    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        roots.push(PathBuf::from(local).join("Android").join("Sdk").join("ndk"));
    }
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        roots.push(home.join("Android").join("Sdk").join("ndk"));
        roots.push(home.join("Library").join("Android").join("sdk").join("ndk"));
    }
    if let Ok(sdk) = std::env::var("ANDROID_HOME") {
        roots.push(PathBuf::from(sdk).join("ndk"));
    }
    if let Ok(sdk) = std::env::var("ANDROID_SDK_ROOT") {
        roots.push(PathBuf::from(sdk).join("ndk"));
    }
    roots.push(PathBuf::from("/opt/android-sdk/ndk"));
    roots.push(PathBuf::from("/usr/lib/android-sdk/ndk"));

    let mut best: Option<(Vec<u64>, PathBuf)> = None;
    for root in roots {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !is_ndk(&path) {
                continue;
            }
            let version = version_key(&path);
            if best.as_ref().map(|(v, _)| version > *v).unwrap_or(true) {
                best = Some((version, path));
            }
        }
    }
    best.map(|(_, path)| path)
}

fn is_ndk(path: &Path) -> bool {
    path.join("toolchains").join("llvm").is_dir()
}

/// Ubah nama folder versi NDK menjadi kunci pembanding (27.0.12077973 → [27,0,12077973]).
fn version_key(path: &Path) -> Vec<u64> {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_abi_has_unique_target_and_clang() {
        let mut targets: Vec<&str> = Vec::new();
        for abi in ABIS {
            assert!(
                !targets.contains(&abi.target),
                "target duplikat: {}",
                abi.target
            );
            targets.push(abi.target);
            assert!(
                abi.clang.starts_with(abi.target.split('-').next().unwrap())
                    || abi.target.starts_with("armv7") && abi.clang.starts_with("armv7a")
            );
        }
    }

    #[test]
    fn version_key_orders_numerically() {
        let old = version_key(Path::new("/ndk/26.1.10909125"));
        let new = version_key(Path::new("/ndk/27.0.12077973"));
        assert!(new > old);
        let patch = version_key(Path::new("/ndk/27.1.0"));
        assert!(patch > new);
    }

    #[test]
    fn min_api_is_numeric() {
        assert!(MIN_API.parse::<u32>().is_ok());
    }
}

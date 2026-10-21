//! Uji integrasi end-to-end: menjalankan biner `kg` sebagai proses terpisah.
//!
//! Tes di sini memverifikasi hal yang tidak terlihat dari unit test: kode
//! keluar, penulisan ke stdout, pembacaan stdin, dan penghormatan variabel
//! lingkungan (mis. `KG_DATA_DIR`).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Jalankan biner dengan argumen dan input stdin tertentu.
fn run(args: &[&str], stdin: &str, envs: &[(&str, &str)]) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kg"));
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NO_COLOR", "1")
        .env("KG_COLUMNS", "88");

    for (key, value) in envs {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn().expect("biner kg bisa dijalankan");
    child
        .stdin
        .as_mut()
        .expect("stdin tersedia")
        .write_all(stdin.as_bytes())
        .expect("tulis stdin");

    let output = child.wait_with_output().expect("tunggu proses");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn temp_data_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("uji-integrasi")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn dashboard_runs_without_args() {
    let dir = temp_data_dir("dasbor");
    let (code, stdout, _) = run(&[], "", &[("KG_DATA_DIR", dir.to_str().unwrap())]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Korean Grammar"));
    assert!(stdout.contains("은/는"));
}

#[test]
fn help_and_version_exit_zero() {
    let (code, stdout, _) = run(&["--help"], "", &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("PENGGUNAAN"));

    let (code, stdout, _) = run(&["--version"], "", &[]);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("korean-rust-grammar "));
}

#[test]
fn unknown_command_exits_with_code_two() {
    let (code, _, stderr) = run(&["tidak-ada"], "", &[]);
    assert_eq!(code, 2);
    assert!(stderr.contains("perintah tidak dikenal"));
}

#[test]
fn list_level_filter_only_shows_that_level() {
    let dir = temp_data_dir("daftar");
    let (code, stdout, _) = run(
        &["list", "--level", "3"],
        "",
        &[("KG_DATA_DIR", dir.to_str().unwrap())],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("-고 / -서"));
    assert!(!stdout.contains("한글 — Huruf Korea"));
}

#[test]
fn rom_command_produces_expected_romanization() {
    let (code, stdout, _) = run(&["rom", "안녕하세요"], "", &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("annyeonghaseyo"));
}

#[test]
fn hangul_command_shows_jamo() {
    let (code, stdout, _) = run(&["hangul", "한"], "", &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("ㅎ"));
    assert!(stdout.contains("ㅏ"));
    assert!(stdout.contains("ㄴ"));
}

#[test]
fn quiz_persists_progress_to_given_directory() {
    let dir = temp_data_dir("kuis");
    // Jawab "1" empat kali — cukup untuk menyelesaikan kuis 3 soal topik 3.
    let (code, stdout, _) = run(
        &["quiz", "3"],
        "1\n1\n1\n1\n",
        &[("KG_DATA_DIR", dir.to_str().unwrap())],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("Ringkasan kuis"));
    assert!(stdout.contains("Progres disimpan"));

    let file = dir.join("progress.json");
    assert!(file.exists(), "berkas progres harus dibuat");

    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("\"eun-neun\""));

    // `kg progress` harus membaca kembali berkas tersebut.
    let (code, stdout, _) = run(&["progress"], "", &[("KG_DATA_DIR", dir.to_str().unwrap())]);
    assert_eq!(code, 0);
    assert!(stdout.contains("은/는"));
    assert!(stdout.contains("topik dikerjakan"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn quiz_quit_leaves_no_summary() {
    let dir = temp_data_dir("kuis-keluar");
    let (code, stdout, _) = run(
        &["quiz", "1"],
        "q\n",
        &[("KG_DATA_DIR", dir.to_str().unwrap())],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("Sesi dihentikan"));
    assert!(!stdout.contains("Ringkasan kuis"));
}

#[test]
fn quiz_hint_does_not_consume_question() {
    let dir = temp_data_dir("kuis-petunjuk");
    let (code, stdout, _) = run(
        &["quiz", "1", "-n", "1"],
        "h\n1\n",
        &[("KG_DATA_DIR", dir.to_str().unwrap())],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("petunjuk:"));
    assert!(stdout.contains("Ringkasan kuis"));
}

#[test]
fn progress_is_empty_message_when_no_data() {
    let dir = temp_data_dir("progres-kosong");
    let (code, stdout, _) = run(&["progress"], "", &[("KG_DATA_DIR", dir.to_str().unwrap())]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Belum ada topik"));
}

#[test]
fn reset_requires_confirmation() {
    let dir = temp_data_dir("reset");
    let (code, _, stderr) = run(&["reset"], "", &[("KG_DATA_DIR", dir.to_str().unwrap())]);
    assert_eq!(code, 2);
    assert!(stderr.contains("--yes"));
}

#[test]
fn reset_clears_saved_progress() {
    let dir = temp_data_dir("reset-ok");
    run(
        &["quiz", "3"],
        "1\n1\n1\n1\n",
        &[("KG_DATA_DIR", dir.to_str().unwrap())],
    );
    let (code, stdout, _) = run(
        &["reset", "--yes"],
        "",
        &[("KG_DATA_DIR", dir.to_str().unwrap())],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("Progres dihapus"));

    let text = std::fs::read_to_string(dir.join("progress.json")).unwrap();
    assert!(text.contains("\"answered\": 0"));
}

#[test]
fn search_without_results_is_still_success() {
    let (code, stdout, _) = run(&["search", "zzzz-tidak-ada"], "", &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Tidak ada hasil"));
}

#[test]
fn show_all_topics_succeeds() {
    for no in 1..=12 {
        let key = no.to_string();
        let (code, stdout, _) = run(&["show", &key], "", &[]);
        assert_eq!(code, 0, "topik {no} harus bisa dibuka");
        assert!(stdout.contains("Topik "), "topik {no} tanpa header");
        assert!(stdout.contains("Latihan: kg quiz"));
    }
}

#[test]
fn random_prints_an_example() {
    let (code, stdout, _) = run(&["random"], "", &[]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Topik "));
}

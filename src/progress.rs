//! Penyimpanan progres belajar dalam berkas JSON.
//!
//! Lokasi berkas mengikuti konvensi tiap OS:
//!
//! | OS      | Lokasi                                                        |
//! |---------|---------------------------------------------------------------|
//! | Linux   | `$XDG_DATA_HOME/kg-korean/progress.json` (default `~/.local/share/...`) |
//! | macOS   | `~/Library/Application Support/kg-korean/progress.json`         |
//! | Windows | `%APPDATA%\kg-korean\progress.json`                             |
//! | Semua   | `$KG_DATA_DIR` bila variabel itu diset                             |
//!
//! Semua operasi I/O mengembalikan `Result`; tidak ada `unwrap` di jalur
//! runtime sehingga progres yang rusak tidak membuat program panik.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::json::Value;

static RESOLVED_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Folder data aplikasi (`…/kg-korean`).
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("KG_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    RESOLVED_DIR.get_or_init(resolve_writable_dir).clone()
}

/// Tentukan folder data yang benar-benar bisa ditulis.
///
/// Urutan percobaan: folder standar OS → `<cwd>/.kg-data`. Bila keduanya
/// gagal (mis. sandbox tanpa izin tulis), tetap kembalikan folder standar agar
/// pesan kesalahan menyebut lokasi yang wajar.
fn resolve_writable_dir() -> PathBuf {
    let preferred = preferred_dir();
    if is_writable(&preferred) {
        return preferred;
    }
    if let Ok(cwd) = std::env::current_dir() {
        let fallback = cwd.join(".kg-data");
        if is_writable(&fallback) {
            return fallback;
        }
    }
    preferred
}

/// Folder data sesuai konvensi OS, tanpa memeriksa izin tulis.
fn preferred_dir() -> PathBuf {
    if cfg!(windows) {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("kg-korean");
        }
        return PathBuf::from(".").join("kg-korean");
    }
    if cfg!(target_os = "macos") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("kg-korean");
        }
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("kg-korean");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("kg-korean");
    }
    PathBuf::from(".").join("kg-korean")
}

/// Cek apakah folder bisa dibuat dan ditulisi.
fn is_writable(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".tulis-uji");
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Lokasi berkas progres.
pub fn progress_path() -> PathBuf {
    data_dir().join("progress.json")
}

/// Statistik satu topik.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TopicStat {
    /// Persentase terbaik (0..=100).
    pub best: u32,
    /// Jumlah jawaban benar pada percobaan terbaik.
    pub best_correct: u32,
    /// Total soal saat percobaan terbaik.
    pub total: u32,
    /// Berapa kali topik dikerjakan.
    pub attempts: u32,
    /// Unix timestamp (detik) percobaan terakhir.
    pub last_at: u64,
}

/// Seluruh progres belajar.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progress {
    pub version: u32,
    pub topics: Vec<(String, TopicStat)>,
    /// Total jawaban yang pernah dijawab.
    pub answered: u32,
}

impl Progress {
    pub const VERSION: u32 = 1;

    pub fn new() -> Progress {
        Progress {
            version: Progress::VERSION,
            topics: Vec::new(),
            answered: 0,
        }
    }

    /// Baca progres dari disk. Berkas tidak ada / rusak → progres kosong.
    pub fn load() -> Progress {
        Self::load_from(&progress_path())
    }

    /// Baca progres dari lokasi tertentu (berguna untuk pengujian).
    pub fn load_from(path: &Path) -> Progress {
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => return Progress::new(),
        };
        Self::from_json(&text).unwrap_or_else(|_| Progress::new())
    }

    /// Simpan ke lokasi default.
    pub fn save(&self) -> io::Result<PathBuf> {
        self.save_to(&progress_path())
    }

    /// Simpan ke lokasi tertentu; membuat folder induk bila perlu.
    pub fn save_to(&self, path: &Path) -> io::Result<PathBuf> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_json_pretty())?;
        Ok(path.to_path_buf())
    }

    /// Ubah progres menjadi JSON (indentasi 2 spasi).
    pub fn to_json_pretty(&self) -> String {
        self.to_value().to_json_pretty(2)
    }

    pub fn to_value(&self) -> Value {
        let mut root = Value::object();
        root.insert("version", self.version)
            .insert("answered", self.answered)
            .insert("topics", Value::object());

        if let Value::Object(map) = &mut root {
            if let Some(Value::Object(topics)) = map.get_mut("topics") {
                for (id, stat) in &self.topics {
                    let mut entry = Value::object();
                    entry
                        .insert("best", stat.best)
                        .insert("bestCorrect", stat.best_correct)
                        .insert("total", stat.total)
                        .insert("attempts", stat.attempts)
                        .insert("lastAt", stat.last_at);
                    topics.insert(id.clone(), entry);
                }
            }
        }
        root
    }

    /// Bangun progres dari teks JSON.
    pub fn from_json(text: &str) -> Result<Progress, String> {
        let root = Value::parse(text).map_err(|e| e.to_string())?;
        Ok(Self::from_value(&root))
    }

    fn from_value(root: &Value) -> Progress {
        let version = root
            .get("version")
            .and_then(Value::as_u32)
            .unwrap_or(Progress::VERSION);
        let answered = root.get("answered").and_then(Value::as_u32).unwrap_or(0);

        let mut topics = Vec::new();
        if let Some(map) = root.get("topics") {
            for (id, stat) in map.entries() {
                topics.push((
                    id.clone(),
                    TopicStat {
                        best: stat.get("best").and_then(Value::as_u32).unwrap_or(0),
                        best_correct: stat.get("bestCorrect").and_then(Value::as_u32).unwrap_or(0),
                        total: stat.get("total").and_then(Value::as_u32).unwrap_or(0),
                        attempts: stat.get("attempts").and_then(Value::as_u32).unwrap_or(0),
                        last_at: stat.get("lastAt").and_then(Value::as_u64).unwrap_or(0),
                    },
                ));
            }
        }
        topics.sort_by(|a, b| a.0.cmp(&b.0));

        Progress {
            version,
            topics,
            answered,
        }
    }

    /// Statistik satu topik (None bila belum pernah dikerjakan).
    pub fn get(&self, topic_id: &str) -> Option<&TopicStat> {
        self.topics
            .iter()
            .find(|(id, _)| id == topic_id)
            .map(|(_, stat)| stat)
    }

    /// Catat hasil satu sesi kuis.
    pub fn record(&mut self, topic_id: &str, correct: u32, total: u32) {
        let percent = if total == 0 {
            0
        } else {
            ((correct as f64 * 100.0) / total as f64).round() as u32
        };
        let now = now_unix();

        match self.topics.iter_mut().find(|(id, _)| id == topic_id) {
            Some((_, stat)) => {
                if percent > stat.best {
                    stat.best = percent;
                    stat.best_correct = correct;
                    stat.total = total;
                }
                stat.attempts = stat.attempts.saturating_add(1);
                stat.last_at = now;
            }
            None => {
                self.topics.push((
                    topic_id.to_string(),
                    TopicStat {
                        best: percent,
                        best_correct: correct,
                        total,
                        attempts: 1,
                        last_at: now,
                    },
                ));
                self.topics.sort_by(|a, b| a.0.cmp(&b.0));
            }
        }
        self.answered = self.answered.saturating_add(total);
    }

    /// Rata-rata skor terbaik dari topik yang sudah dikerjakan.
    pub fn average(&self) -> Option<u32> {
        if self.topics.is_empty() {
            return None;
        }
        let sum: u32 = self.topics.iter().map(|(_, s)| s.best).sum();
        Some((sum as f64 / self.topics.len() as f64).round() as u32)
    }

    /// Jumlah topik yang sudah pernah dikerjakan.
    pub fn completed_topic_count(&self) -> usize {
        self.topics.len()
    }

    /// Hapus semua progres.
    pub fn reset(&mut self) {
        self.topics.clear();
        self.answered = 0;
    }
}

/// Waktu sekarang sebagai Unix timestamp (detik).
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Format Unix timestamp menjadi `YYYY-MM-DD HH:MM` (UTC) tanpa crate waktu.
pub fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "—".to_string();
    }
    let days = (ts / 86_400) as i64;
    let secs_of_day = ts % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        y,
        m,
        d,
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60
    )
}

/// Algoritma Howard Hinnant: hari sejak 1970-01-01 → (tahun, bulan, hari).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // 0..146096
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // 0..399
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // 0..365
    let mp = (5 * doy + 2) / 153; // 0..11
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let mut p = Progress::new();
        p.record("hangeul", 3, 4);
        p.record("eun-neun", 2, 3);
        let text = p.to_json_pretty();
        let back = Progress::from_json(&text).unwrap();
        assert_eq!(back.topics.len(), 2);
        assert_eq!(back.answered, 7);
        assert_eq!(back.get("hangeul").unwrap().best, 75);
    }

    #[test]
    fn best_score_only_increases() {
        let mut p = Progress::new();
        p.record("x", 4, 4);
        p.record("x", 1, 4);
        let stat = p.get("x").unwrap();
        assert_eq!(stat.best, 100);
        assert_eq!(stat.best_correct, 4);
        assert_eq!(stat.attempts, 2);
    }

    #[test]
    fn corrupt_json_falls_back_to_empty() {
        let back = Progress::from_json("ini bukan json").unwrap_err();
        assert!(back.contains("byte"));
    }

    #[test]
    fn average_and_count() {
        let mut p = Progress::new();
        assert_eq!(p.average(), None);
        p.record("a", 1, 2); // 50
        p.record("b", 3, 4); // 75
        assert_eq!(p.average(), Some(63)); // (50+75)/2 = 62.5 → 63
        assert_eq!(p.completed_topic_count(), 2);
    }

    #[test]
    fn save_and_load_via_temp_path() {
        // Gunakan folder target agar tidak bergantung pada izin folder temp OS.
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("uji-progres");
        let path = dir.join("progress.json");
        let _ = fs::remove_dir_all(&dir);

        let mut p = Progress::new();
        p.record("hangeul", 2, 4);
        let written = p.save_to(&path).expect("tulis progres");
        assert_eq!(written, path);

        let loaded = Progress::load_from(&path);
        assert_eq!(loaded.get("hangeul").unwrap().best, 50);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_is_empty() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("uji-tidak-ada")
            .join("progress.json");
        let p = Progress::load_from(&path);
        assert!(p.topics.is_empty());
        assert_eq!(p.version, Progress::VERSION);
    }

    #[test]
    fn formats_epoch_correctly() {
        assert_eq!(format_timestamp(0), "—");
        assert_eq!(format_timestamp(1_700_000_000), "2023-11-14 22:13");
    }

    #[test]
    fn reset_clears_everything() {
        let mut p = Progress::new();
        p.record("a", 1, 1);
        p.reset();
        assert!(p.topics.is_empty());
        assert_eq!(p.answered, 0);
    }
}

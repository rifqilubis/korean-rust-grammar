//! Tipe data untuk bank materi tata bahasa Korea.
//!
//! Semua data bersifat `'static` (dibekukan saat kompilasi), jadi tidak ada
//! alokasi saat runtime untuk memuat materi.

/// Tingkat kesulitan topik.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Dasar,
    Menengah,
    Lanjutan,
}

impl Level {
    pub const ALL: [Level; 3] = [Level::Dasar, Level::Menengah, Level::Lanjutan];

    pub fn label(self) -> &'static str {
        match self {
            Level::Dasar => "Dasar",
            Level::Menengah => "Menengah",
            Level::Lanjutan => "Lanjutan",
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Level::Dasar => 1,
            Level::Menengah => 2,
            Level::Lanjutan => 3,
        }
    }

    pub fn from_code(code: u8) -> Option<Level> {
        match code {
            1 => Some(Level::Dasar),
            2 => Some(Level::Menengah),
            3 => Some(Level::Lanjutan),
            _ => None,
        }
    }
}

/// Satu baris tabel aturan (tiga kolom).
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub a: &'static str,
    pub b: &'static str,
    pub c: &'static str,
}

/// Satu contoh kalimat: Korea, romanisasi, terjemahan.
#[derive(Clone, Copy, Debug)]
pub struct Example {
    pub ko: &'static str,
    pub rom: &'static str,
    pub id: &'static str,
}

/// Satu butir soal pilihan ganda.
#[derive(Clone, Copy, Debug)]
pub struct QuizItem {
    pub q: &'static str,
    pub options: [&'static str; 4],
    pub answer: usize,
    pub why: &'static str,
}

/// Blok penyusun halaman materi.
#[derive(Clone, Copy, Debug)]
pub enum Block {
    Heading(&'static str),
    Text(&'static str),
    Formula(&'static str),
    Rules(&'static [Rule]),
    Examples(&'static [Example]),
    Note(&'static str),
}

/// Satu topik pelajaran lengkap.
#[derive(Clone, Copy, Debug)]
pub struct Topic {
    pub id: &'static str,
    pub no: u32,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub level: Level,
    pub icon: &'static str,
    pub intro: &'static str,
    pub blocks: &'static [Block],
    pub quiz: &'static [QuizItem],
}

impl Topic {
    /// Judul gabungan, mis. `은/는 — Partikel topik`.
    pub fn full_title(&self) -> String {
        format!("{} — {}", self.title, self.subtitle)
    }

    /// Nomor topik dalam format dua digit.
    pub fn number(&self) -> String {
        format!("{:02}", self.no)
    }

    pub fn quiz_count(&self) -> usize {
        self.quiz.len()
    }

    /// Semua teks pada topik ini (untuk pencarian).
    pub fn plain_text(&self) -> Vec<&'static str> {
        let mut out = vec![self.title, self.subtitle, self.intro];
        for block in self.blocks {
            match block {
                Block::Heading(s) | Block::Text(s) | Block::Formula(s) | Block::Note(s) => {
                    out.push(s)
                }
                Block::Rules(rules) => {
                    for r in rules.iter() {
                        out.push(r.a);
                        out.push(r.b);
                        out.push(r.c);
                    }
                }
                Block::Examples(items) => {
                    for e in items.iter() {
                        out.push(e.ko);
                        out.push(e.rom);
                        out.push(e.id);
                    }
                }
            }
        }
        for item in self.quiz {
            out.push(item.q);
            out.push(item.why);
        }
        out
    }

    /// Gabungan teks huruf kecil untuk pencarian cepat.
    pub fn haystack(&self) -> String {
        self.plain_text().join(" \u{1} ").to_lowercase()
    }

    /// Apakah topik cocok dengan kata kunci (case-insensitive).
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return false;
        }
        self.haystack().contains(&q)
    }
}

/* -------------------------------------------------------------- helper const */

/// Pembuat [`Rule`] ringkas untuk dipakai di `static` data.
pub const fn r(a: &'static str, b: &'static str, c: &'static str) -> Rule {
    Rule { a, b, c }
}

/// Pembuat [`Example`] ringkas.
pub const fn e(ko: &'static str, rom: &'static str, id: &'static str) -> Example {
    Example { ko, rom, id }
}

/// Pembuat [`QuizItem`] ringkas.
pub const fn q(
    question: &'static str,
    options: [&'static str; 4],
    answer: usize,
    why: &'static str,
) -> QuizItem {
    QuizItem {
        q: question,
        options,
        answer,
        why,
    }
}

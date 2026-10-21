//! Manipulasi 한글 (Hangul): dekomposisi, komposisi, dan romanisasi.
//!
//! Suku kata 한글 modern berbentuk blok dengan rumus:
//!
//! ```text
//! U+AC00 + (초성 * 21 + 중성) * 28 + 종성
//! ```
//!
//! Modul ini menyediakan ketiga tabel tersebut, sanggup memecah kalimat apa pun
//! menjadi suku kata, lalu merangkainya kembali, serta mengubahnya menjadi
//! huruf Latin dengan aturan Romanisasi Korea yang disederhanakan.

/// Jumlah konsonan awal (초성), termasuk ㅇ yang tidak berbunyi.
pub const LEAD_COUNT: usize = 19;
/// Jumlah vokal (중성).
pub const VOWEL_COUNT: usize = 21;
/// Jumlah konsonan akhir (종성); indeks 0 berarti "tanpa konsonan akhir".
pub const TAIL_COUNT: usize = 28;

const SYLLABLE_BASE: u32 = 0xAC00;
const SYLLABLE_LAST: u32 = 0xD7A3;

/// Hasil dekomposisi satu suku kata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Syllable {
    /// Indeks konsonan awal (0..19).
    pub lead: usize,
    /// Indeks vokal (0..21).
    pub vowel: usize,
    /// Indeks konsonan akhir (0..28); 0 = tidak ada.
    pub tail: usize,
}

impl Syllable {
    /// Buat suku kata baru bila indeks valid.
    pub fn new(lead: usize, vowel: usize, tail: usize) -> Option<Syllable> {
        if lead < LEAD_COUNT && vowel < VOWEL_COUNT && tail < TAIL_COUNT {
            Some(Syllable { lead, vowel, tail })
        } else {
            None
        }
    }

    /// Nama/simbol jamo, mis. `("ㄱ", "ㅏ", "")`.
    pub fn jamo(self) -> (&'static str, &'static str, &'static str) {
        (
            LEAD_JAMO[self.lead],
            VOWEL_JAMO[self.vowel],
            TAIL_JAMO[self.tail],
        )
    }

    /// Apakah suku kata ini punya konsonan akhir (받침).
    pub fn has_batchim(self) -> bool {
        self.tail != 0
    }
}

/// Simbol jamo konsonan awal, untuk ditampilkan di UI.
pub const LEAD_JAMO: [&str; LEAD_COUNT] = [
    "ㄱ", "ㄲ", "ㄴ", "ㄷ", "ㄸ", "ㄹ", "ㅁ", "ㅂ", "ㅃ", "ㅅ", "ㅆ", "ㅇ", "ㅈ", "ㅉ", "ㅊ", "ㅋ",
    "ㅌ", "ㅍ", "ㅎ",
];

/// Simbol jamo vokal.
pub const VOWEL_JAMO: [&str; VOWEL_COUNT] = [
    "ㅏ", "ㅐ", "ㅑ", "ㅒ", "ㅓ", "ㅔ", "ㅕ", "ㅖ", "ㅗ", "ㅘ", "ㅙ", "ㅚ", "ㅛ", "ㅜ", "ㅝ", "ㅞ",
    "ㅟ", "ㅠ", "ㅡ", "ㅢ", "ㅣ",
];

/// Simbol jamo konsonan akhir (indeks 0 = kosong).
pub const TAIL_JAMO: [&str; TAIL_COUNT] = [
    "", "ㄱ", "ㄲ", "ㄳ", "ㄴ", "ㄵ", "ㄶ", "ㄷ", "ㄹ", "ㄺ", "ㄻ", "ㄼ", "ㄽ", "ㄾ", "ㄿ", "ㅀ",
    "ㅁ", "ㅂ", "ㅄ", "ㅅ", "ㅆ", "ㅇ", "ㅈ", "ㅊ", "ㅋ", "ㅌ", "ㅍ", "ㅎ",
];

/// Romanisasi konsonan awal.
const LEAD_ROM: [&str; LEAD_COUNT] = [
    "g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s", "ss", "", "j", "jj", "ch", "k", "t", "p",
    "h",
];

/// Romanisasi vokal.
const VOWEL_ROM: [&str; VOWEL_COUNT] = [
    "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa", "wae", "oe", "yo", "u", "wo", "we",
    "wi", "yu", "eu", "ui", "i",
];

/// Romanisasi konsonan akhir (dibaca sendiri, tanpa suku kata berikutnya).
const TAIL_ROM: [&str; TAIL_COUNT] = [
    "", "k", "k", "k", "n", "n", "n", "t", "l", "k", "m", "l", "l", "l", "p", "l", "m", "p", "p",
    "t", "t", "ng", "t", "t", "k", "t", "p", "t",
];

/// Aturan 연음 per 받침: `(bunyi yang TETAP di suku kata pertama, konsonan
/// yang PINDAH menjadi 초성 suku kata berikutnya)`.
///
/// * `ㅇ` tidak pernah berpindah — 학생이 dibaca `haksaengi`, bukan `haksaei`.
/// * Gugus konsonan dipecah: 없어 → `eop` + `sseo`, 값이 → `gap` + `ssi`.
/// * `ㅎ` akhir hilang saat bertemu vokal: 좋아 → `joa`.
const TAIL_SPLIT: [(&str, Option<usize>); TAIL_COUNT] = [
    ("", None),      // (kosong)
    ("", Some(0)),   // ㄱ → g
    ("", Some(1)),   // ㄲ → kk
    ("k", Some(9)),  // ㄳ → k + s
    ("", Some(2)),   // ㄴ → n
    ("n", Some(12)), // ㄵ → n + j
    ("n", None),     // ㄶ → n (ㅎ hilang)
    ("", Some(3)),   // ㄷ → d
    ("", Some(5)),   // ㄹ → r
    ("l", Some(0)),  // ㄺ → l + g
    ("l", Some(6)),  // ㄻ → l + m
    ("l", Some(7)),  // ㄼ → l + b
    ("l", Some(9)),  // ㄽ → l + s
    ("l", Some(16)), // ㄾ → l + t
    ("l", Some(17)), // ㄿ → l + p
    ("l", None),     // ㅀ → l (ㅎ hilang)
    ("", Some(6)),   // ㅁ → m
    ("", Some(7)),   // ㅂ → b
    ("p", Some(9)),  // ㅄ → p + s
    ("", Some(9)),   // ㅅ → s
    ("", Some(10)),  // ㅆ → ss
    ("ng", None),    // ㅇ → ng (TIDAK pindah)
    ("", Some(12)),  // ㅈ → j
    ("", Some(14)),  // ㅊ → ch
    ("", Some(15)),  // ㅋ → k
    ("", Some(16)),  // ㅌ → t
    ("", Some(17)),  // ㅍ → p
    ("", None),      // ㅎ → hilang
];

/// Kelompok konsonan akhir untuk asimilasi hidung (비음화).
fn nasal_class(tail: usize) -> Option<u8> {
    match tail {
        1 | 2 | 9 | 24 => Some(b'k'),                  // ㄱ ㄲ ㄺ ㅋ -> ng
        7 | 19 | 20 | 22 | 23 | 25 | 27 => Some(b't'), // ㄷ ㅅ ㅆ ㅈ ㅊ ㅌ ㅎ -> n
        17 | 18 | 26 | 11 | 14 => Some(b'p'),          // ㅂ ㅄ ㅍ ㄼ ㄿ -> m
        _ => None,
    }
}

/// Apakah karakter merupakan suku kata 한글 yang lengkap.
pub fn is_syllable(ch: char) -> bool {
    (SYLLABLE_BASE..=SYLLABLE_LAST).contains(&(ch as u32))
}

/// Apakah karakter termasuk jamo (konsonan/vokal lepas).
pub fn is_jamo(ch: char) -> bool {
    matches!(ch as u32, 0x3131..=0x318E | 0x1100..=0x11FF)
}

/// Pecah satu suku kata menjadi (초성, 중성, 종성).
pub fn decompose(ch: char) -> Option<Syllable> {
    if !is_syllable(ch) {
        return None;
    }
    let index = ch as u32 - SYLLABLE_BASE;
    let tail = (index % 28) as usize;
    let vowel = ((index / 28) % 21) as usize;
    let lead = (index / 28 / 21) as usize;
    Some(Syllable { lead, vowel, tail })
}

/// Rangkai kembali (초성, 중성, 종성) menjadi suku kata.
pub fn compose(syl: Syllable) -> Option<char> {
    if syl.lead >= LEAD_COUNT || syl.vowel >= VOWEL_COUNT || syl.tail >= TAIL_COUNT {
        return None;
    }
    let code = SYLLABLE_BASE
        + ((syl.lead as u32 * VOWEL_COUNT as u32 + syl.vowel as u32) * TAIL_COUNT as u32
            + syl.tail as u32);
    char::from_u32(code)
}

/// Ubah seluruh teks menjadi daftar suku kata.
pub fn syllables(text: &str) -> Vec<Syllable> {
    text.chars().filter_map(decompose).collect()
}

/// Hitung jumlah suku kata 한글 pada teks.
pub fn count_syllables(text: &str) -> usize {
    text.chars().filter(|c| is_syllable(*c)).count()
}

/// Ubah ke huruf latin (romanisasi yang disederhanakan).
///
/// Aturan yang diterapkan:
/// * 연음 — 받침 pindah ke posisi awal bila suku kata berikutnya berawal vokal,
///   dengan gugus konsonan (ㅄ, ㄳ, …) dipecah dan ㅇ tetap di tempatnya;
/// * 비음화 — asimilasi hidung `ㄱ/ㄷ/ㅂ + ㄴ/ㅁ` (국물 → gungmul);
/// * huruf non-한글 dipertahankan apa adanya.
///
/// ```
/// use korean_rust_grammar::hangul::romanize;
///
/// assert_eq!(romanize("한국어"), "hangugeo");   // 연음
/// assert_eq!(romanize("학생이"), "haksaengi");  // ㅇ tidak berpindah
/// assert_eq!(romanize("없어요"), "eopseoyo");   // gugus ㅄ dipecah
/// assert_eq!(romanize("국물"), "gungmul");      // 비음화
/// ```
pub fn romanize(text: &str) -> String {
    enum Tok {
        Hangul(Syllable),
        Other(char),
    }

    let mut toks: Vec<Tok> = text
        .chars()
        .map(|c| match decompose(c) {
            Some(s) => Tok::Hangul(s),
            None => Tok::Other(c),
        })
        .collect();

    let mut out = String::with_capacity(text.len() * 2);
    let mut i = 0;

    while i < toks.len() {
        let syl = match toks[i] {
            Tok::Other(c) => {
                out.push(c);
                i += 1;
                continue;
            }
            Tok::Hangul(s) => s,
        };

        let next = match toks.get(i + 1) {
            Some(Tok::Hangul(n)) => Some(*n),
            _ => None,
        };

        if let Some(n) = next {
            // 연음: 받침 bertemu suku kata berawal ㅇ.
            if syl.tail != 0 && n.lead == 11 {
                let (stays, moves) = TAIL_SPLIT[syl.tail];
                out.push_str(LEAD_ROM[syl.lead]);
                out.push_str(VOWEL_ROM[syl.vowel]);
                out.push_str(stays);
                // Suku kata berikutnya menyerap konsonan yang berpindah.
                toks[i + 1] = Tok::Hangul(Syllable {
                    lead: moves.unwrap_or(11),
                    vowel: n.vowel,
                    tail: n.tail,
                });
                i += 1;
                continue;
            }

            // 비음화: ㄱ/ㄷ/ㅂ + ㄴ/ㅁ.
            if syl.tail != 0 && (n.lead == 2 || n.lead == 6) {
                if let Some(class) = nasal_class(syl.tail) {
                    let tail = match class {
                        b'k' => 21, // ㅇ
                        b't' => 4,  // ㄴ
                        _ => 16,    // ㅁ
                    };
                    out.push_str(LEAD_ROM[syl.lead]);
                    out.push_str(VOWEL_ROM[syl.vowel]);
                    out.push_str(TAIL_ROM[tail]);
                    i += 1;
                    continue;
                }
            }
        }

        out.push_str(LEAD_ROM[syl.lead]);
        out.push_str(VOWEL_ROM[syl.vowel]);
        out.push_str(TAIL_ROM[syl.tail]);
        i += 1;
    }

    out
}

/// Rasio karakter 한글 terhadap seluruh karakter non-spasi (0.0 - 1.0).
pub fn hangul_ratio(text: &str) -> f64 {
    let mut total = 0usize;
    let mut hangul = 0usize;
    for ch in text.chars() {
        if ch.is_whitespace() || ch.is_ascii_punctuation() {
            continue;
        }
        total += 1;
        if is_syllable(ch) {
            hangul += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        hangul as f64 / total as f64
    }
}

/// Pecah teks menjadi pasangan (suku kata, romanisasinya).
pub fn annotate(text: &str) -> Vec<(char, String)> {
    text.chars()
        .filter(|c| is_syllable(*c))
        .map(|c| (c, romanize(&c.to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_and_compose_roundtrip() {
        // Seluruh 11.172 suku kata 한글 modern harus bolak-balik tanpa cacat.
        let mut checked = 0usize;
        for code in SYLLABLE_BASE..=SYLLABLE_LAST {
            let ch = char::from_u32(code).expect("kode valid");
            let syl = decompose(ch).expect("suku kata valid");
            assert_eq!(compose(syl), Some(ch), "gagal pada U+{:04X}", code);
            checked += 1;
        }
        assert_eq!(checked, 11_172);
    }

    #[test]
    fn known_offsets() {
        let han = decompose('한').unwrap();
        assert_eq!(han.lead, 18); // ㅎ
        assert_eq!(han.vowel, 0); // ㅏ
        assert_eq!(han.tail, 4); // ㄴ
        assert!(han.has_batchim());
    }

    #[test]
    fn romanizes_basic_words() {
        assert_eq!(romanize("한글"), "hangeul");
        assert_eq!(romanize("한국어"), "hangugeo");
        assert_eq!(romanize("김치"), "gimchi");
        assert_eq!(romanize("사람"), "saram");
    }

    #[test]
    fn resyllabification_moves_batchim() {
        // 옷 + 이 -> o + si
        assert_eq!(romanize("옷이"), "osi");
        // 밥 + 을 -> ba + beul
        assert_eq!(romanize("밥을"), "babeul");
    }

    #[test]
    fn nasal_assimilation() {
        // 국 + 물 -> gungmul
        assert_eq!(romanize("국물"), "gungmul");
        // 한 + 번 -> hanbeon (ㄴ + ㅂ tidak berasimilasi)
        assert_eq!(romanize("한번"), "hanbeon");
    }

    #[test]
    fn keeps_non_hangul() {
        // 예 = ㅇ+ㅖ → "ye", 요 = ㅇ+ㅛ → "yo"
        assert_eq!(romanize("저는 Minsu예요!"), "jeoneun Minsuyeyo!");
    }

    #[test]
    fn counts_and_ratio() {
        assert_eq!(count_syllables("저는 학생이에요"), 7);
        assert!(hangul_ratio("저는 학생이에요") > 0.99);
        assert_eq!(hangul_ratio("hello"), 0.0);
    }

    #[test]
    fn annotate_pairs() {
        let pairs = annotate("물");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, '물');
        assert_eq!(pairs[0].1, "mul");
    }

    #[test]
    fn batchim_ieung_never_moves() {
        // ㅇ tetap di tempatnya: 학생이 → haksaengi, bukan haksaei.
        assert_eq!(romanize("학생이"), "haksaengi");
        assert_eq!(romanize("학생이에요"), "haksaengieyo");
        assert_eq!(romanize("강이"), "gangi");
    }

    #[test]
    fn consonant_clusters_split_on_resyllabification() {
        // Gugus ㅄ/ㄳ/ㄵ dipecah: satu bunyi tinggal, satu pindah.
        assert_eq!(romanize("없어요"), "eopseoyo");
        assert_eq!(romanize("값이"), "gapsi");
        assert_eq!(romanize("앉아요"), "anjayo");
        assert_eq!(romanize("읽어요"), "ilgeoyo");
    }

    #[test]
    fn final_h_drops_before_vowel() {
        assert_eq!(romanize("좋아요"), "joayo");
    }

    #[test]
    fn every_example_matches_its_manual_romanization() {
        // Menjaga agar data materi dan mesin romanisasi tidak pernah menyimpang.
        let clean = |s: &str| -> String {
            s.chars()
                .filter(|c| !c.is_ascii_punctuation() && !c.is_whitespace())
                .collect::<String>()
                .to_lowercase()
        };
        for topic in crate::data::TOPICS {
            for block in topic.blocks {
                if let crate::model::Block::Examples(items) = block {
                    for item in items.iter() {
                        assert_eq!(
                            clean(item.rom),
                            clean(&romanize(item.ko)),
                            "romanisasi '{}' pada topik {} tidak cocok",
                            item.ko,
                            topic.id
                        );
                    }
                }
            }
        }
    }
}

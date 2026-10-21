//! Utilitas tampilan terminal: warna ANSI, lebar CJK, tabel, dan panel.
//!
//! Semua fungsi di sini sadar lebar tampilan (wide char): suku kata 한글 dan
//! emoji dihitung dua kolom sehingga tabel tetap rata di terminal mana pun.

use std::io::{self, Write};

/* ------------------------------------------------------------------ warna */

/// Kode warna ANSI sederhana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Grey,
    Dim,
    Bold,
    Reset,
}

impl Color {
    pub fn code(self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[97m",
            Color::Grey => "\x1b[90m",
            Color::Dim => "\x1b[2m",
            Color::Bold => "\x1b[1m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Apakah warna boleh dipakai pada stdout.
///
/// Warna dimatikan bila: variabel `NO_COLOR` diset, `TERM=dumb`, atau stdout
/// bukan terminal (mis. saat output di-pipe ke berkas).
pub fn color_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if matches!(std::env::var("TERM").as_deref(), Ok("dumb")) {
        return false;
    }
    if std::env::var_os("KG_FORCE_COLOR").is_some() {
        return true;
    }
    is_tty()
}

#[cfg(windows)]
fn is_tty() -> bool {
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}

#[cfg(not(windows))]
fn is_tty() -> bool {
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}

/// Beri warna pada teks bila `enabled`.
pub fn paint(text: &str, color: Color, enabled: bool) -> String {
    if enabled {
        format!("{}{}{}", color.code(), text, Color::Reset.code())
    } else {
        text.to_string()
    }
}

/// Gabungkan beberapa gaya (mis. Bold + Cyan).
pub fn paint_many(text: &str, codes: &[Color], enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let mut prefix = String::new();
    for c in codes {
        prefix.push_str(c.code());
    }
    format!("{}{}{}", prefix, text, Color::Reset.code())
}

/* -------------------------------------------------------------- lebar CJK */

/// Lebar satu karakter di terminal: 2 untuk CJK/한글/emoji, 0 untuk gabungan.
pub fn char_width(ch: char) -> usize {
    let cp = ch as u32;
    if cp == 0 {
        return 0;
    }
    // Zero-width / kombinasi.
    if matches!(cp, 0x0300..=0x036F | 0x200B..=0x200F | 0xFE00..=0xFE0F | 0xFEFF) {
        return 0;
    }
    // Blok CJK, 한글, Hiragana/Katakana, dan emoji.
    if matches!(cp,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE6F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD
    ) {
        return 2;
    }
    1
}

/// Lebar tampilan sebuah teks (mengabaikan kode ANSI).
pub fn display_width(text: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    for ch in text.chars() {
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        width += char_width(ch);
    }
    width
}

/// Potong teks agar muat `max` kolom tanpa memotong karakter.
pub fn truncate(text: &str, max: usize) -> String {
    if display_width(text) <= max {
        return text.to_string();
    }
    let mut out = String::new();
    let mut width = 0;
    // Sisakan satu kolom untuk elipsis.
    let limit = max.saturating_sub(1);
    for ch in text.chars() {
        let w = char_width(ch);
        if width + w > limit {
            break;
        }
        out.push(ch);
        width += w;
    }
    out.push('…');
    out
}

/// Bungkus teks menjadi beberapa baris selebar `width` kolom.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_width = 0;

    for word in text.split_whitespace() {
        let word_width = display_width(word);
        let space = if line.is_empty() { 0 } else { 1 };
        if line_width + space + word_width > width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            line_width = 0;
        }
        if !line.is_empty() {
            line.push(' ');
            line_width += 1;
        }
        line.push_str(word);
        line_width += word_width;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Tambah spasi kanan agar lebar tampilan = `width`.
pub fn pad_right(text: &str, width: usize) -> String {
    let current = display_width(text);
    if current >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - current))
    }
}

/// Tambah spasi kiri agar lebar tampilan = `width`.
pub fn pad_left(text: &str, width: usize) -> String {
    let current = display_width(text);
    if current >= width {
        text.to_string()
    } else {
        format!("{}{}", " ".repeat(width - current), text)
    }
}

/* ------------------------------------------------------------------ panel */

/// Gaya kotak.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Border {
    Rounded,
    Heavy,
    None,
}

impl Border {
    fn chars(
        self,
    ) -> (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ) {
        match self {
            Border::Rounded => ("╭", "╮", "╰", "╯", "─", "│"),
            Border::Heavy => ("┏", "┓", "┗", "┛", "━", "┃"),
            Border::None => ("", "", "", "", "", ""),
        }
    }
}

/// Cetak kotak berisi baris-baris yang sudah diformat.
pub fn panel<W: Write>(
    out: &mut W,
    title: &str,
    lines: &[String],
    width: usize,
    border: Border,
) -> io::Result<()> {
    let (tl, tr, bl, br, h, v) = border.chars();
    let inner = width.saturating_sub(4);

    if !tl.is_empty() {
        if title.is_empty() {
            writeln!(out, "{}{}{}", tl, h.repeat(inner + 2), tr)?;
        } else {
            let head = format!(" {} ", truncate(title, inner.saturating_sub(2)));
            let head_width = display_width(&head);
            // 1 kolom untuk garis horizontal pembuka sebelum judul.
            let fill = (inner + 2).saturating_sub(head_width + 1);
            writeln!(out, "{}{}{}{}{}", tl, h, head, h.repeat(fill), tr)?;
        }
    }

    for line in lines {
        let text = truncate(line, inner);
        let pad = inner.saturating_sub(display_width_ansi(&text));
        if v.is_empty() {
            writeln!(out, "{}", text)?;
        } else {
            writeln!(out, "{} {}{} {}", v, text, " ".repeat(pad), v)?;
        }
    }

    if !bl.is_empty() {
        writeln!(out, "{}{}{}", bl, h.repeat(inner + 2), br)?;
    }
    Ok(())
}

/// Lebar tampilan yang paham ANSI (nama eksplisit untuk keterbacaan).
fn display_width_ansi(text: &str) -> usize {
    display_width(text)
}

/* ------------------------------------------------------------------ tabel */

/// Tabel tiga kolom sederhana untuk daftar aturan.
pub fn table3<W: Write>(
    out: &mut W,
    headers: (&str, &str, &str),
    rows: &[(&str, &str, &str)],
) -> io::Result<()> {
    let mut w1 = display_width(headers.0);
    let mut w2 = display_width(headers.1);
    let mut w3 = display_width(headers.2);
    for (a, b, c) in rows {
        w1 = w1.max(display_width(a));
        w2 = w2.max(display_width(b));
        w3 = w3.max(display_width(c));
    }

    writeln!(
        out,
        "  {}  {}  {}",
        pad_right(headers.0, w1),
        pad_right(headers.1, w2),
        headers.2
    )?;
    writeln!(
        out,
        "  {}  {}  {}",
        "─".repeat(w1),
        "─".repeat(w2),
        "─".repeat(w3)
    )?;
    for (a, b, c) in rows {
        writeln!(out, "  {}  {}  {}", pad_right(a, w1), pad_right(b, w2), c)?;
    }
    Ok(())
}

/// Progress bar berbasis teks, mis. `████████░░░░ 62%`.
pub fn progress_bar(percent: u32, width: usize) -> String {
    let percent = percent.min(100);
    let filled = ((width * percent as usize) + 50) / 100;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Lebar terminal, dengan fallback 80 kolom.
pub fn terminal_width() -> usize {
    if let Some(cols) = std::env::var("KG_COLUMNS")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
    {
        return cols.clamp(40, 200);
    }
    #[cfg(windows)]
    {
        if let Some(cols) = windows_columns() {
            return cols.clamp(40, 200);
        }
    }
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(n) = cols.trim().parse::<usize>() {
            return n.clamp(40, 200);
        }
    }
    80
}

#[cfg(windows)]
fn windows_columns() -> Option<usize> {
    // `mode con` adalah cara paling portabel tanpa crate WinAPI.
    let output = std::process::Command::new("cmd")
        .args(["/C", "mode con"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("columns") || lower.contains("kolom") {
            if let Some(num) = line
                .split_whitespace()
                .rev()
                .find(|p| p.parse::<usize>().is_ok())
            {
                return num.parse::<usize>().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hangul_is_double_width() {
        assert_eq!(char_width('한'), 2);
        assert_eq!(char_width('a'), 1);
        assert_eq!(display_width("한글"), 4);
        assert_eq!(display_width("abc"), 3);
    }

    #[test]
    fn ansi_is_ignored_in_width() {
        let painted = paint("abc", Color::Red, true);
        assert_eq!(display_width(&painted), 3);
    }

    #[test]
    fn paint_respects_flag() {
        assert_eq!(paint("x", Color::Red, false), "x");
        assert!(paint("x", Color::Red, true).contains("\x1b[31m"));
    }

    #[test]
    fn truncate_handles_wide_chars() {
        let s = truncate("한국어문법", 5);
        assert!(display_width(&s) <= 5);
        assert!(s.ends_with('…'));
        assert_eq!(truncate("abc", 10), "abc");
    }

    #[test]
    fn padding_matches_display_width() {
        assert_eq!(display_width(&pad_right("한", 6)), 6);
        assert_eq!(display_width(&pad_left("한", 6)), 6);
        assert_eq!(pad_right("abcdef", 3), "abcdef");
    }

    #[test]
    fn wrap_splits_long_text() {
        let lines = wrap("satu dua tiga empat lima enam", 10);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(display_width(line) <= 10, "baris terlalu lebar: {line}");
        }
    }

    #[test]
    fn progress_bar_bounds() {
        assert_eq!(progress_bar(0, 10), "░".repeat(10));
        assert_eq!(progress_bar(100, 10), "█".repeat(10));
        assert_eq!(progress_bar(150, 4).chars().count(), 4);
    }

    #[test]
    fn panel_writes_borders() {
        let mut buf: Vec<u8> = Vec::new();
        panel(
            &mut buf,
            "Judul",
            &["isi 한글".to_string()],
            30,
            Border::Rounded,
        )
        .unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains('╭'));
        assert!(text.contains("Judul"));
        assert!(text.contains('╰'));
    }

    #[test]
    fn table_renders_all_rows() {
        let mut buf: Vec<u8> = Vec::new();
        table3(&mut buf, ("A", "B", "C"), &[("ㄱ", "g", "k")]).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("ㄱ"));
        assert_eq!(text.lines().count(), 3);
    }
}

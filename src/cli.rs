//! Antarmuka baris perintah (CLI) untuk aplikasi korean-rust-grammar.
//!
//! Modul ini memisahkan parsing argumen + tampilan dari [`crate::main`] agar
//! mudah diuji dan dipakai ulang. Seluruh input/output melewati parameter
//! `input`/`output` sehingga bisa diuji tanpa terminal sungguhan.

use std::io::{BufRead, Write};

use crate::data;
use crate::hangul;
use crate::model::{Block, Level, Topic};
use crate::progress::{format_timestamp, Progress};
use crate::quiz::{Question, Session, Verdict};
use crate::rng::Rng;
use crate::ui::{self, Border, Color};
use crate::VERSION;

/// Konfigurasi tampilan yang dipakai sepanjang satu eksekusi.
pub struct Ctx {
    pub color: bool,
    pub width: usize,
}

impl Ctx {
    pub fn detect() -> Ctx {
        Ctx {
            color: ui::color_enabled(),
            width: ui::terminal_width(),
        }
    }

    fn title(&self, text: &str) -> String {
        ui::paint_many(text, &[Color::Bold, Color::Cyan], self.color)
    }

    fn accent(&self, text: &str) -> String {
        ui::paint(text, Color::Yellow, self.color)
    }

    fn dim(&self, text: &str) -> String {
        ui::paint(text, Color::Grey, self.color)
    }

    fn ok(&self, text: &str) -> String {
        ui::paint(text, Color::Green, self.color)
    }

    fn bad(&self, text: &str) -> String {
        ui::paint(text, Color::Red, self.color)
    }
}

/// Hasil eksekusi yang memengaruhi kode keluar proses.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    UserError(String),
}

const HELP: &str = "\
korean-rust-grammar — belajar tata bahasa Korea dari terminal

PENGGUNAAN
  kg                       Dasbor: daftar topik + progres
  kg list [--level <n>]    Daftar semua topik (1=dasar, 2=menengah, 3=lanjutan)
  kg show <topik>          Tampilkan materi lengkap satu topik
  kg quiz <topik> [-n N]   Kuis pilihan ganda interaktif
  kg search <kata>         Cari kata kunci di seluruh materi
  kg rom <teks>            Romanisasi teks 한글
  kg hangul <teks>         Uraikan teks menjadi 초성/중성/종성
  kg random                Tampilkan satu contoh kalimat acak
  kg progress              Lihat progres belajar
  kg reset --yes           Hapus semua progres
  kg help                  Tampilan bantuan ini

OPSI
  -n, --jumlah <N>         Batasi jumlah soal kuis
      --level <1|2|3>      Filter tingkat pada 'kg list'
      --no-color           Matikan warna ANSI
  -V, --version            Cetak versi
  -h, --help               Cetak bantuan

CONTOH
  kg show 3
  kg quiz eun-neun -n 5
  kg search hujan
  kg rom 안녕하세요
  kg hangul 한국어

LINGKUNGAN
  NO_COLOR=1               Matikan warna
  KG_COLUMNS=100           Paksa lebar tampilan
  KG_DATA_DIR=./data       Ubah lokasi berkas progres
";

/// Titik masuk CLI. `args` tidak termasuk nama program.
pub fn run<R: BufRead, W: Write>(args: &[String], input: &mut R, out: &mut W) -> Outcome {
    let mut ctx = Ctx::detect();
    let mut args: Vec<String> = args.to_vec();

    // Opsi global bisa muncul di mana saja.
    if args.iter().any(|a| a == "--no-color") {
        ctx.color = false;
        args.retain(|a| a != "--no-color");
    }

    let Some(command) = args.first().cloned() else {
        return dashboard(&ctx, out);
    };
    let rest = &args[1..];

    match command.as_str() {
        "help" | "-h" | "--help" => {
            let _ = writeln!(out, "{}", HELP);
            Outcome::Ok
        }
        "-V" | "--version" | "version" => {
            let _ = writeln!(out, "korean-rust-grammar {}", VERSION);
            Outcome::Ok
        }
        "list" | "ls" => list_command(&ctx, rest, out),
        "show" | "materi" => show_command(&ctx, rest, out),
        "quiz" | "kuis" => quiz_command(&ctx, rest, input, out),
        "search" | "cari" => search_command(&ctx, rest, out),
        "rom" | "romanize" => rom_command(&ctx, rest, out),
        "hangul" | "urai" => hangul_command(&ctx, rest, out),
        "random" | "acak" => random_command(&ctx, out),
        "progress" | "progres" => progress_command(&ctx, out),
        "reset" => reset_command(&ctx, rest, out),
        other => Outcome::UserError(format!(
            "perintah tidak dikenal: '{}'. Jalankan 'kg help'.",
            other
        )),
    }
}

/* ------------------------------------------------------------- dasbor */

fn dashboard<W: Write>(ctx: &Ctx, out: &mut W) -> Outcome {
    let progress = Progress::load();
    let total_quiz = data::total_quiz();
    let done = progress.completed_topic_count();

    let mut lines = Vec::new();
    lines.push(format!(
        "{} topik · {} soal · progres {}",
        data::TOPICS.len(),
        total_quiz,
        if done == 0 {
            ctx.dim("belum dimulai")
        } else {
            ctx.ok(&format!("{}/{} topik", done, data::TOPICS.len()))
        }
    ));
    lines.push(String::new());
    lines.push(ctx.dim("Tingkat  No  Topik"));
    for topic in data::TOPICS {
        let status = match progress.get(topic.id) {
            Some(stat) => {
                let bar = ui::progress_bar(stat.best, 10);
                format!("{} {:>3}%", bar, stat.best)
            }
            None => ctx.dim("belum dikerjakan"),
        };
        lines.push(format!(
            "{}  {}  {} {}",
            ctx.dim(&format!("[{}]", topic.level.code())),
            topic.number(),
            ui::pad_right(&ui::truncate(&topic.full_title(), 34), 36),
            status
        ));
    }
    lines.push(String::new());
    lines.push(ctx.dim("Mulai: kg show 1   ·   Kuis: kg quiz 1   ·   Semua perintah: kg help"));

    let width = ctx.width.min(84);
    let _ = ui::panel(
        out,
        "한국어 문법 · Korean Grammar",
        &lines,
        width,
        Border::Rounded,
    );
    Outcome::Ok
}

/* --------------------------------------------------------------- list */

fn list_command<W: Write>(ctx: &Ctx, args: &[String], out: &mut W) -> Outcome {
    let mut level_filter: Option<Level> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--level" {
            let Some(value) = args.get(i + 1) else {
                return Outcome::UserError("--level butuh nilai 1, 2, atau 3".into());
            };
            match value.trim().parse::<u8>().ok().and_then(Level::from_code) {
                Some(level) => level_filter = Some(level),
                None => return Outcome::UserError(format!("tingkat tidak valid: '{}'", value)),
            }
            i += 2;
            continue;
        }
        // Bentuk `--level=2`
        if let Some(value) = args[i].strip_prefix("--level=") {
            match value.parse::<u8>().ok().and_then(Level::from_code) {
                Some(level) => level_filter = Some(level),
                None => return Outcome::UserError(format!("tingkat tidak valid: '{}'", value)),
            }
            i += 1;
            continue;
        }
        return Outcome::UserError(format!("argumen tidak dikenal: '{}'", args[i]));
    }

    let progress = Progress::load();
    let topics: Vec<&'static Topic> = match level_filter {
        Some(level) => data::by_level(level),
        None => data::TOPICS.iter().collect(),
    };

    let header = match level_filter {
        Some(level) => format!("Topik tingkat {}", level.label()),
        None => "Semua topik".to_string(),
    };
    let _ = writeln!(out, "{}", ctx.title(&header));
    let _ = writeln!(out, "{}", ctx.dim(&"─".repeat(ctx.width.min(70))));

    for topic in &topics {
        let status = match progress.get(topic.id) {
            Some(stat) => ctx.ok(&format!("{:>3}%", stat.best)),
            None => ctx.dim("   —"),
        };
        let _ = writeln!(
            out,
            "{} {}  {}  {}  {}",
            ctx.accent(&format!("[{}]", topic.level.code())),
            topic.number(),
            ui::pad_right(&ui::truncate(&topic.full_title(), 38), 40),
            ui::pad_right(&format!("{} soal", topic.quiz_count()), 8),
            status
        );
        let _ = writeln!(
            out,
            "     {}",
            ctx.dim(&ui::truncate(topic.intro, ctx.width.min(70) - 8))
        );
    }

    let _ = writeln!(
        out,
        "\n{}",
        ctx.dim(&format!(
            "{} topik ditampilkan · 'kg show <no>' untuk membaca",
            topics.len()
        ))
    );
    Outcome::Ok
}

/* --------------------------------------------------------------- show */

fn show_command<W: Write>(ctx: &Ctx, args: &[String], out: &mut W) -> Outcome {
    let Some(key) = args.first() else {
        return Outcome::UserError("pakai: kg show <id|nomor>".into());
    };
    let Some(topic) = data::find(key) else {
        return Outcome::UserError(format!("topik '{}' tidak ditemukan. Coba 'kg list'.", key));
    };

    let header = format!("{}  {}", topic.icon, topic.full_title());
    let _ = writeln!(out, "{}", ctx.title(&header));
    let _ = writeln!(
        out,
        "{}",
        ctx.dim(&format!(
            "Topik {} · tingkat {} · {} soal",
            topic.number(),
            topic.level.label(),
            topic.quiz_count()
        ))
    );
    let _ = writeln!(out);
    write_wrapped(ctx, out, topic.intro, 0);
    let _ = writeln!(out);

    for block in topic.blocks {
        match block {
            Block::Heading(text) => {
                let _ = writeln!(out, "\n{}", ctx.title(&format!("▌{}", text)));
            }
            Block::Text(text) => {
                write_wrapped(ctx, out, text, 2);
                let _ = writeln!(out);
            }
            Block::Formula(formula) => {
                let _ = writeln!(
                    out,
                    "  {} {}",
                    ctx.accent("➜"),
                    ui::paint(formula, Color::Cyan, ctx.color)
                );
                let _ = writeln!(out);
            }
            Block::Note(text) => {
                let lines = ui::wrap(text, ctx.width.min(78).saturating_sub(8));
                let _ = ui::panel(out, "Catatan", &lines, ctx.width.min(78), Border::Rounded);
            }
            Block::Rules(rules) => {
                // Lebar kolom dihitung dari lebar tampilan (한글 = 2 kolom).
                let mut w1 = ui::display_width("bentuk");
                let mut w2 = ui::display_width("romanisasi");
                for rule in rules.iter() {
                    w1 = w1.max(ui::display_width(rule.a));
                    w2 = w2.max(ui::display_width(rule.b));
                }
                let _ = writeln!(
                    out,
                    "  {}  {}  {}",
                    ui::pad_right(&ctx.dim("bentuk"), w1),
                    ui::pad_right(&ctx.dim("romanisasi"), w2),
                    ctx.dim("arti")
                );
                for rule in rules.iter() {
                    let _ = writeln!(
                        out,
                        "  {}  {}  {}",
                        ui::pad_right(&ui::paint(rule.a, Color::Yellow, ctx.color), w1),
                        ui::pad_right(rule.b, w2),
                        rule.c
                    );
                }
                let _ = writeln!(out);
            }
            Block::Examples(items) => {
                for item in items.iter() {
                    let rom = if item.rom.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", ctx.dim(item.rom))
                    };
                    let _ = writeln!(out, "  {} {}{}", ctx.accent("•"), item.ko, rom);
                    let _ = writeln!(out, "    {}", ctx.dim(item.id));
                }
            }
        }
    }

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{}",
        ctx.dim(&format!(
            "Latihan: kg quiz {} — {} soal siap dikerjakan",
            topic.no,
            topic.quiz_count()
        ))
    );
    Outcome::Ok
}

fn write_wrapped<W: Write>(ctx: &Ctx, out: &mut W, text: &str, indent: usize) {
    let width = ctx.width.min(78).saturating_sub(indent);
    for line in ui::wrap(text, width.max(20)) {
        let _ = writeln!(out, "{}{}", " ".repeat(indent), line);
    }
}

/* --------------------------------------------------------------- quiz */

fn quiz_command<R: BufRead, W: Write>(
    ctx: &Ctx,
    args: &[String],
    input: &mut R,
    out: &mut W,
) -> Outcome {
    let mut key: Option<String> = None;
    let mut limit = 0usize;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--jumlah" => {
                let Some(value) = args.get(i + 1) else {
                    return Outcome::UserError("-n butuh angka".into());
                };
                match value.trim().parse::<usize>() {
                    Ok(n) if n > 0 => limit = n,
                    _ => {
                        return Outcome::UserError(format!("jumlah soal tidak valid: '{}'", value))
                    }
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                return Outcome::UserError(format!("opsi tidak dikenal: '{}'", other));
            }
            other => {
                if key.is_none() {
                    key = Some(other.to_string());
                } else {
                    return Outcome::UserError(format!("argumen berlebih: '{}'", other));
                }
                i += 1;
            }
        }
    }

    let Some(key) = key else {
        return Outcome::UserError("pakai: kg quiz <id|nomor> [-n jumlah]".into());
    };
    let Some(topic) = data::find(&key) else {
        return Outcome::UserError(format!("topik '{}' tidak ditemukan. Coba 'kg list'.", key));
    };

    let mut rng = Rng::from_time();
    let mut session = Session::new(topic, limit, &mut rng);

    let _ = writeln!(
        out,
        "{}  {} soal",
        ctx.title(&format!("Kuis · {}", topic.full_title())),
        session.total()
    );
    let _ = writeln!(
        out,
        "{}",
        ctx.dim("Jawab dengan 1-4 atau a-d. 'q' untuk keluar, 'h' untuk petunjuk.")
    );

    let mut quit = false;
    while let Some(question) = session.current().cloned() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{} {}",
            ctx.accent(&format!("[{}/{}]", question.number, session.total())),
            question.text
        );
        for (index, option) in question.options.iter().enumerate() {
            let _ = writeln!(out, "   {} {}", Question::letter(index), option);
        }
        let _ = write!(out, "{} ", ctx.dim("›"));
        let _ = out.flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                // EOF: hentikan sesi.
                let _ = writeln!(out);
                quit = true;
                break;
            }
            Ok(_) => {}
            Err(e) => return Outcome::UserError(format!("gagal membaca input: {}", e)),
        }

        let trimmed = line.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "q" || trimmed == "quit" || trimmed == "keluar" {
            quit = true;
            break;
        }
        if trimmed == "h" || trimmed == "hint" || trimmed == "petunjuk" {
            let _ = writeln!(out, "   {} {}", ctx.dim("petunjuk:"), question.why);
            continue;
        }

        let Some(choice) = parse_choice(&trimmed) else {
            let _ = writeln!(out, "   {}", ctx.bad("masukkan 1-4 atau a-d"));
            continue;
        };

        match session.answer(choice) {
            Some(Verdict::Correct) => {
                let _ = writeln!(out, "   {} {}", ctx.ok("✓ benar!"), ctx.dim(question.why));
            }
            Some(Verdict::Wrong) => {
                let _ = writeln!(
                    out,
                    "   {} jawaban benar: {}",
                    ctx.bad("✗ salah."),
                    ctx.accent(question.correct_text())
                );
                let _ = writeln!(out, "   {}", ctx.dim(question.why));
            }
            None => break,
        }
    }

    if quit {
        let _ = writeln!(
            out,
            "\n{}",
            ctx.dim(&format!(
                "Sesi dihentikan. Skor sementara: {}/{} ({}%)",
                session.correct,
                session.correct + session.wrong,
                session.percent()
            ))
        );
        return Outcome::Ok;
    }

    print_summary(ctx, out, &session);
    let _ = out.flush();
    Outcome::Ok
}

fn parse_choice(text: &str) -> Option<usize> {
    match text {
        "1" | "a" => Some(0),
        "2" | "b" => Some(1),
        "3" | "c" => Some(2),
        "4" | "d" => Some(3),
        _ => None,
    }
}

fn print_summary<W: Write>(ctx: &Ctx, out: &mut W, session: &Session) {
    let total = session.correct + session.wrong;
    let percent = session.percent();

    let _ = writeln!(out);
    let _ = writeln!(out, "{}", ctx.title("Ringkasan kuis"));
    let _ = writeln!(
        out,
        "  benar {} · salah {} · total {}",
        ctx.ok(&session.correct.to_string()),
        ctx.bad(&session.wrong.to_string()),
        total
    );
    let _ = writeln!(
        out,
        "  {} {}%  {}",
        ui::progress_bar(percent, 24),
        percent,
        ctx.accent(session.grade())
    );

    if !session.mistakes.is_empty() {
        let _ = writeln!(out, "\n{}", ctx.dim("Yang perlu diulang:"));
        for (i, mistake) in session.mistakes.iter().enumerate() {
            let _ = writeln!(out, "  {}. {}", i + 1, mistake.question);
            let _ = writeln!(
                out,
                "     {} {}",
                ctx.dim("jawabanmu:"),
                ctx.bad(&mistake.chosen)
            );
            let _ = writeln!(
                out,
                "     {} {}",
                ctx.dim("seharusnya:"),
                ctx.ok(&mistake.correct)
            );
            let _ = writeln!(out, "     {}", ctx.dim(mistake.why));
        }
    }

    let mut progress = Progress::load();
    progress.record(session.topic_id, session.correct as u32, total as u32);
    match progress.save() {
        Ok(path) => {
            let _ = writeln!(
                out,
                "\n{}",
                ctx.dim(&format!("Progres disimpan di {}", path.display()))
            );
        }
        Err(e) => {
            let _ = writeln!(
                out,
                "\n{}",
                ctx.bad(&format!("Gagal menyimpan progres: {}", e))
            );
        }
    }
}

/* -------------------------------------------------------------- search */

fn search_command<W: Write>(ctx: &Ctx, args: &[String], out: &mut W) -> Outcome {
    if args.is_empty() {
        return Outcome::UserError("pakai: kg search <kata kunci>".into());
    }
    let query = args.join(" ");
    let hits = data::search(&query);
    if hits.is_empty() {
        let _ = writeln!(
            out,
            "{}",
            ctx.bad(&format!("Tidak ada hasil untuk '{}'.", query))
        );
        return Outcome::Ok;
    }

    let _ = writeln!(
        out,
        "{} {}",
        ctx.title("Hasil pencarian:"),
        ctx.dim(&format!("'{}' · {} topik", query, hits.len()))
    );
    for topic in hits {
        let _ = writeln!(
            out,
            "  {} {}  {}",
            ctx.accent(&topic.number()),
            topic.full_title(),
            ctx.dim(&format!("({} soal)", topic.quiz_count()))
        );
        // Tampilkan satu contoh kalimat yang memuat kata kunci bila ada.
        let needle = query.to_lowercase();
        for block in topic.blocks {
            if let Block::Examples(items) = block {
                for item in items.iter() {
                    let hay = format!("{} {} {}", item.ko, item.rom, item.id).to_lowercase();
                    if hay.contains(&needle) {
                        let _ = writeln!(out, "      {} {}", ctx.dim("→"), item.ko);
                        break;
                    }
                }
            }
        }
    }
    Outcome::Ok
}

/* ------------------------------------------------------------ romanisasi */

fn rom_command<W: Write>(ctx: &Ctx, args: &[String], out: &mut W) -> Outcome {
    if args.is_empty() {
        return Outcome::UserError("pakai: kg rom <teks 한글>".into());
    }
    let text = args.join(" ");
    if hangul::count_syllables(&text) == 0 {
        return Outcome::UserError("teks tidak memuat suku kata 한글".into());
    }
    let romanized = hangul::romanize(&text);
    let _ = writeln!(out, "{} {}", ctx.dim("한글:"), text);
    let _ = writeln!(out, "{} {}", ctx.dim("rom :"), ctx.accent(&romanized));
    let _ = writeln!(
        out,
        "{} {:.0}% suku kata 한글 · {}",
        ctx.dim("info:"),
        hangul::hangul_ratio(&text) * 100.0,
        ctx.dim(&format!("{} suku kata", hangul::count_syllables(&text)))
    );
    Outcome::Ok
}

/* --------------------------------------------------------------- hangul */

fn hangul_command<W: Write>(ctx: &Ctx, args: &[String], out: &mut W) -> Outcome {
    if args.is_empty() {
        return Outcome::UserError("pakai: kg hangul <teks 한글>".into());
    }
    let text = args.join(" ");
    let pairs = hangul::annotate(&text);
    if pairs.is_empty() {
        return Outcome::UserError("teks tidak memuat suku kata 한글".into());
    }

    let _ = writeln!(out, "{}", ctx.title("Uraian suku kata"));
    let header = ["suku", "초성", "중성", "종성", "bunyi"];
    let mut widths = [0usize; 5];
    for (i, h) in header.iter().enumerate() {
        widths[i] = ui::display_width(h);
    }
    let mut rows: Vec<[String; 5]> = Vec::new();
    for (ch, rom) in pairs.iter() {
        let Some(syl) = hangul::decompose(*ch) else {
            continue;
        };
        let (lead, vowel, tail) = syl.jamo();
        let row = [
            ch.to_string(),
            lead.to_string(),
            vowel.to_string(),
            if tail.is_empty() {
                "—".to_string()
            } else {
                tail.to_string()
            },
            rom.clone(),
        ];
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(ui::display_width(cell));
        }
        rows.push(row);
    }

    let _ = writeln!(
        out,
        "  {}",
        header
            .iter()
            .enumerate()
            .map(|(i, h)| ui::pad_right(&ctx.dim(h), widths[i]))
            .collect::<Vec<_>>()
            .join("  ")
    );
    for row in rows {
        let _ = writeln!(
            out,
            "  {}",
            row.iter()
                .enumerate()
                .map(|(i, cell)| ui::pad_right(cell, widths[i]))
                .collect::<Vec<_>>()
                .join("  ")
        );
    }
    let _ = writeln!(
        out,
        "\n{} {}",
        ctx.dim("romanisasi:"),
        hangul::romanize(&text)
    );
    Outcome::Ok
}

/* --------------------------------------------------------------- random */

fn random_command<W: Write>(ctx: &Ctx, out: &mut W) -> Outcome {
    let mut rng = Rng::from_time();
    let topic = data::TOPICS[rng.below(data::TOPICS.len())];
    let examples: Vec<_> = topic
        .blocks
        .iter()
        .flat_map(|b| match b {
            Block::Examples(items) => items.to_vec(),
            _ => Vec::new(),
        })
        .collect();
    if examples.is_empty() {
        return Outcome::Ok;
    }
    let item = examples[rng.below(examples.len())];
    let _ = writeln!(
        out,
        "{}",
        ctx.dim(&format!("Topik {} · {}", topic.no, topic.full_title()))
    );
    let _ = writeln!(
        out,
        "  {}",
        ui::paint_many(item.ko, &[Color::Bold, Color::Yellow], ctx.color)
    );
    let _ = writeln!(out, "  {}", ctx.dim(item.rom));
    let _ = writeln!(out, "  {}", item.id);
    Outcome::Ok
}

/* ------------------------------------------------------------- progres */

fn progress_command<W: Write>(ctx: &Ctx, out: &mut W) -> Outcome {
    let progress = Progress::load();
    let path = crate::progress::progress_path();

    let _ = writeln!(out, "{}", ctx.title("Progres belajar"));
    let _ = writeln!(out, "{}", ctx.dim(&format!("Berkas: {}", path.display())));

    if progress.topics.is_empty() {
        let _ = writeln!(
            out,
            "\n{}",
            ctx.dim("Belum ada topik yang dikerjakan. Mulai dengan 'kg quiz 1'.")
        );
        return Outcome::Ok;
    }

    let _ = writeln!(out);
    for (id, stat) in &progress.topics {
        let title = data::find(id)
            .map(|t| t.full_title())
            .unwrap_or_else(|| id.clone());
        let _ = writeln!(
            out,
            "  {} {} {:>3}%  {}",
            ui::progress_bar(stat.best, 16),
            ui::pad_right(&title, 30),
            stat.best,
            ctx.dim(&format!(
                "{}x · terakhir {}",
                stat.attempts,
                format_timestamp(stat.last_at)
            ))
        );
    }

    let answered = progress.answered;
    let average = progress.average().unwrap_or(0);
    let _ = writeln!(
        out,
        "\n  {} topik dikerjakan · {} jawaban · rata-rata {}%",
        progress.completed_topic_count(),
        answered,
        average
    );

    let remaining: Vec<&str> = data::TOPICS
        .iter()
        .filter(|t| progress.get(t.id).is_none())
        .map(|t| t.id)
        .collect();
    if !remaining.is_empty() {
        let _ = writeln!(
            out,
            "  {}",
            ctx.dim(&format!("Belum dikerjakan: {}", remaining.join(", ")))
        );
    }
    Outcome::Ok
}

/* --------------------------------------------------------------- reset */

fn reset_command<W: Write>(ctx: &Ctx, args: &[String], out: &mut W) -> Outcome {
    let confirmed = args.iter().any(|a| a == "--yes" || a == "-y");
    if !confirmed {
        return Outcome::UserError(
            "pakai: kg reset --yes (perintah ini menghapus semua progres)".into(),
        );
    }
    let mut progress = Progress::load();
    progress.reset();
    match progress.save() {
        Ok(path) => {
            let _ = writeln!(
                out,
                "{}",
                ctx.ok(&format!("Progres dihapus: {}", path.display()))
            );
            Outcome::Ok
        }
        Err(e) => Outcome::UserError(format!("gagal menyimpan: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_cli(args: &[&str]) -> (Outcome, String) {
        let mut input: &[u8] = b"";
        let mut out: Vec<u8> = Vec::new();
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let outcome = {
            let mut ctx = Ctx {
                color: false,
                width: 80,
            };
            // Opsi --no-color ditangani di run(); shortcut di sini.
            let _ = &mut ctx;
            run(&owned, &mut input, &mut out)
        };
        (outcome, String::from_utf8_lossy(&out).to_string())
    }

    #[test]
    fn help_lists_commands() {
        let (outcome, text) = run_cli(&["help"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains("kg quiz"));
        assert!(text.contains("kg show"));
    }

    #[test]
    fn version_prints_number() {
        let (outcome, text) = run_cli(&["--version"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains(VERSION));
    }

    #[test]
    fn unknown_command_is_user_error() {
        let (outcome, _) = run_cli(&["meledak"]);
        assert!(matches!(outcome, Outcome::UserError(_)));
    }

    #[test]
    fn list_prints_topics() {
        let (outcome, text) = run_cli(&["list"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains("은/는"));
        assert!(text.contains("한글"));
    }

    #[test]
    fn list_invalid_level_fails() {
        let (outcome, _) = run_cli(&["list", "--level", "9"]);
        assert!(matches!(outcome, Outcome::UserError(_)));
    }

    #[test]
    fn show_renders_blocks() {
        let (outcome, text) = run_cli(&["show", "3"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains("은/는"));
        assert!(text.contains("Catatan"));
    }

    #[test]
    fn show_unknown_topic_fails() {
        let (outcome, _) = run_cli(&["show", "99"]);
        assert!(matches!(outcome, Outcome::UserError(_)));
    }

    #[test]
    fn rom_command_romanizes() {
        let (outcome, text) = run_cli(&["rom", "한국어"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains("hangugeo"));
    }

    #[test]
    fn hangul_command_decomposes() {
        let (outcome, text) = run_cli(&["hangul", "한"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains("ㅎ"));
        assert!(text.contains("ㅏ"));
    }

    #[test]
    fn search_finds_topic() {
        let (outcome, text) = run_cli(&["search", "hujan"]);
        assert_eq!(outcome, Outcome::Ok);
        assert!(text.contains("이/가") || text.contains("비"));
    }

    #[test]
    fn quiz_answers_through_stdin() {
        // Jawab "1" berulang; hasil apa pun harus tetap menyelesaikan sesi.
        let mut input: &[u8] = b"1\n1\n1\n1\n1\n1\n";
        let mut out: Vec<u8> = Vec::new();
        let args: Vec<String> = ["quiz", "hangeul", "-n", "2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let outcome = run(&args, &mut input, &mut out);
        assert_eq!(outcome, Outcome::Ok);
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("Ringkasan kuis"));
    }

    #[test]
    fn quiz_quit_early() {
        let mut input: &[u8] = b"q\n";
        let mut out: Vec<u8> = Vec::new();
        let args: Vec<String> = ["quiz", "1"].iter().map(|s| s.to_string()).collect();
        let outcome = run(&args, &mut input, &mut out);
        assert_eq!(outcome, Outcome::Ok);
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("Sesi dihentikan"));
    }

    #[test]
    fn parse_choice_maps_letters() {
        assert_eq!(parse_choice("a"), Some(0));
        assert_eq!(parse_choice("4"), Some(3));
        assert_eq!(parse_choice("z"), None);
    }
}

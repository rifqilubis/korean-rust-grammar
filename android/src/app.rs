//! Layar-layar aplikasi: daftar topik, materi, kuis, hasil, dan progres.
//!
//! Seluruh isi materi serta logika kuis berasal dari pustaka
//! `korean_rust_grammar` — berkas ini hanya mengurus tampilan dan alur layar,
//! sehingga perilakunya identik dengan versi CLI.

use eframe::egui::{self, Align, Button, Layout, Margin, RichText, ScrollArea, Sense, Ui};

use korean_rust_grammar::data;
use korean_rust_grammar::model::{Block, Topic};
use korean_rust_grammar::progress::{format_timestamp, Progress};
use korean_rust_grammar::quiz::{Question, Session, Verdict};
use korean_rust_grammar::rng::Rng;

use crate::theme::{self, Mode, Palette};

/// Layar yang sedang ditampilkan.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Home,
    Topic(&'static str),
    Quiz,
    Result,
    Progress,
}

/// Satu jawaban yang sudah dinilai, ditampilkan sebelum lanjut ke soal berikutnya.
struct Feedback {
    verdict: Verdict,
    chosen: usize,
    correct: usize,
    why: &'static str,
}

pub struct App {
    mode: Mode,
    /// Mode yang paletnya sudah terpasang, agar `apply` tidak dipanggil tiap bingkai.
    applied_mode: Option<Mode>,
    screen: Screen,
    progress: Progress,
    session: Option<Session>,
    feedback: Option<Feedback>,
    /// Minta daftar digulir ke atas saat layar berganti.
    scroll_top: bool,
    /// Pesan kecil di bawah (mis. gagal menyimpan progres).
    toast: Option<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_context(&cc.egui_ctx)
    }

    /// Bangun aplikasi dengan konteks `egui` apa pun (dipakai juga oleh uji).
    pub fn with_context(ctx: &egui::Context) -> Self {
        crate::font::install(ctx);

        let mode = Mode::Dark;
        theme::apply(ctx, mode.palette());

        App {
            mode,
            applied_mode: Some(mode),
            screen: Screen::Home,
            progress: Progress::load(),
            session: None,
            feedback: None,
            scroll_top: true,
            toast: None,
        }
    }

    fn p(&self) -> &'static Palette {
        self.mode.palette()
    }

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.scroll_top = true;
        self.feedback = None;
    }

    fn start_quiz(&mut self, topic: &'static Topic, limit: usize) {
        let mut rng = Rng::from_time();
        self.session = Some(Session::new(topic, limit, &mut rng));
        self.feedback = None;
        self.go(Screen::Quiz);
    }

    /// Simpan progres ke disk; kegagalan hanya diberitahukan, tidak mematikan app.
    fn persist(&mut self) {
        match self.progress.save() {
            Ok(_) => self.toast = None,
            Err(e) => self.toast = Some(format!("Gagal menyimpan progres: {e}")),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

impl App {
    /// Gambar satu bingkai penuh. Dipisah dari `eframe::App::ui` agar bisa
    /// dijalankan tanpa jendela saat pengujian.
    pub fn draw(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();

        // Tema hanya dipasang ulang saat benar-benar berubah.
        if self.applied_mode != Some(self.mode) {
            theme::apply(&ctx, self.p());
            self.applied_mode = Some(self.mode);
        }

        let p = self.p();

        // ---------------------------------------------------------- bilah atas
        egui::Panel::top("bar")
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(p.surface).inner_margin(Margin {
                left: 12,
                right: 12,
                top: 8,
                bottom: 8,
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.screen != Screen::Home
                        && ui
                            .add(
                                Button::new(RichText::new("←").size(20.0))
                                    .min_size(egui::vec2(44.0, 40.0)),
                            )
                            .clicked()
                    {
                        self.go(Screen::Home);
                    }

                    let title = match &self.screen {
                        Screen::Home => "한국어 문법",
                        Screen::Topic(id) => data::find(id).map(|t| t.title).unwrap_or("Materi"),
                        Screen::Quiz => "Kuis",
                        Screen::Result => "Hasil",
                        Screen::Progress => "Progres",
                    };
                    ui.label(RichText::new(title).size(19.0).strong().color(p.text));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let glyph = if self.mode.is_dark() { "☀" } else { "☾" };
                        if ui
                            .add(
                                Button::new(RichText::new(glyph).size(18.0))
                                    .min_size(egui::vec2(44.0, 40.0)),
                            )
                            .clicked()
                        {
                            self.mode = self.mode.toggled();
                        }
                        if self.screen != Screen::Progress
                            && ui
                                .add(
                                    Button::new(RichText::new("진도").size(13.0))
                                        .min_size(egui::vec2(54.0, 40.0)),
                                )
                                .clicked()
                        {
                            self.go(Screen::Progress);
                        }
                    });
                });
            });

        // --------------------------------------------------------------- toast
        if let Some(message) = self.toast.clone() {
            egui::Panel::bottom("toast")
                .show_separator_line(false)
                .frame(
                    egui::Frame::NONE
                        .fill(p.surface)
                        .inner_margin(Margin::same(10)),
                )
                .show(ui, |ui| {
                    ui.colored_label(p.warn, message);
                });
        }

        // ---------------------------------------------------------------- isi
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.bg))
            .show(ui, |ui| {
                let mut area = ScrollArea::vertical().auto_shrink([false, false]);
                if self.scroll_top {
                    area = area.vertical_scroll_offset(0.0);
                    self.scroll_top = false;
                }
                area.show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(Margin {
                            left: 12,
                            right: 12,
                            top: 10,
                            bottom: 24,
                        })
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            match self.screen.clone() {
                                Screen::Home => self.home(ui),
                                Screen::Topic(id) => self.topic(ui, id),
                                Screen::Quiz => self.quiz(ui),
                                Screen::Result => self.result(ui),
                                Screen::Progress => self.progress_screen(ui),
                            }
                        });
                });
            });
    }
}

/* ------------------------------------------------------------------ layar */

impl App {
    fn home(&mut self, ui: &mut Ui) {
        let p = self.p();
        let done = self.progress.completed_topic_count();
        let total = data::TOPICS.len();

        ui.label(
            RichText::new(format!(
                "{total} topik · {} soal · {done}/{total} dikerjakan",
                data::total_quiz()
            ))
            .size(12.0)
            .color(p.text_dim),
        );
        ui.add_space(8.0);

        if let Some(avg) = self.progress.average() {
            theme::card_alt(p).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Rata-rata skor").size(13.0).color(p.text_dim));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{avg}%"))
                                .size(15.0)
                                .strong()
                                .color(p.accent),
                        );
                    });
                });
                ui.add(
                    egui::ProgressBar::new(avg as f32 / 100.0)
                        .desired_width(f32::INFINITY)
                        .fill(p.accent),
                );
            });
            ui.add_space(10.0);
        }

        let mut open: Option<&'static str> = None;
        for topic in data::TOPICS {
            let best = self.progress.get(topic.id).map(|s| s.best);

            let card = theme::card(p).show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Huruf Korea besar sebagai ikon topik.
                    ui.label(RichText::new(topic.icon).size(28.0).color(p.accent));
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(topic.full_title())
                                .size(15.0)
                                .strong()
                                .color(p.text),
                        );
                        ui.label(
                            RichText::new(format!(
                                "Topik {:02} · {} · {} soal",
                                topic.no,
                                topic.level.label(),
                                topic.quiz_count()
                            ))
                            .size(11.0)
                            .color(p.text_dim),
                        );
                    });
                });

                if let Some(best) = best {
                    ui.add_space(6.0);
                    ui.add(
                        egui::ProgressBar::new(best as f32 / 100.0)
                            .desired_width(f32::INFINITY)
                            .fill(p.ok)
                            .text(RichText::new(format!("{best}%")).size(11.0)),
                    );
                }
            });

            if card.response.interact(Sense::click()).clicked() {
                open = Some(topic.id);
            }
            ui.add_space(9.0);
        }

        if let Some(id) = open {
            self.go(Screen::Topic(id));
        }
    }

    fn topic(&mut self, ui: &mut Ui, id: &'static str) {
        let p = self.p();
        let Some(topic) = data::find(id) else {
            ui.label("Topik tidak ditemukan.");
            return;
        };

        ui.label(
            RichText::new(format!(
                "Topik {:02} · {} · {} soal",
                topic.no,
                topic.level.label(),
                topic.quiz_count()
            ))
            .size(11.0)
            .color(p.text_dim),
        );
        ui.add_space(6.0);
        ui.label(RichText::new(topic.intro).size(14.0).color(p.text));

        if let Some(stat) = self.progress.get(topic.id) {
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "Skor terbaik {}% · {}× · {}",
                    stat.best,
                    stat.attempts,
                    format_timestamp(stat.last_at)
                ))
                .size(11.0)
                .color(p.text_dim),
            );
        }
        ui.add_space(8.0);

        for block in topic.blocks {
            self.render_block(ui, block);
        }

        ui.add_space(6.0);
        let start = ui.add(
            Button::new(
                RichText::new(format!("Mulai kuis — {} soal", topic.quiz_count())).size(16.0),
            )
            .min_size(egui::vec2(ui.available_width(), 50.0))
            .fill(p.accent),
        );
        if start.clicked() {
            self.start_quiz(topic, 0);
        }
        ui.add_space(10.0);
    }

    fn render_block(&self, ui: &mut Ui, block: &Block) {
        let p = self.p();
        match block {
            Block::Heading(text) => {
                ui.add_space(8.0);
                ui.label(RichText::new(*text).size(17.0).strong().color(p.accent));
                ui.add_space(3.0);
            }
            Block::Text(text) => {
                ui.label(RichText::new(*text).size(14.0).color(p.text));
                ui.add_space(6.0);
            }
            Block::Formula(formula) => {
                theme::card_alt(p).show(ui, |ui| {
                    ui.label(RichText::new(*formula).size(13.0).monospace().color(p.warn));
                });
                ui.add_space(8.0);
            }
            Block::Note(text) => {
                theme::card_alt(p).show(ui, |ui| {
                    ui.label(
                        RichText::new("Catatan")
                            .size(11.0)
                            .strong()
                            .color(p.text_dim),
                    );
                    ui.label(RichText::new(*text).size(13.0).color(p.text));
                });
                ui.add_space(8.0);
            }
            Block::Rules(rules) => {
                theme::card(p).show(ui, |ui| {
                    egui::Grid::new("aturan")
                        .num_columns(3)
                        .spacing(egui::vec2(10.0, 7.0))
                        .striped(true)
                        .show(ui, |ui| {
                            for rule in rules.iter() {
                                ui.label(RichText::new(rule.a).size(13.0).strong().color(p.text));
                                ui.label(RichText::new(rule.b).size(12.0).color(p.text_dim));
                                ui.label(RichText::new(rule.c).size(12.0).color(p.text_dim));
                                ui.end_row();
                            }
                        });
                });
                ui.add_space(8.0);
            }
            Block::Examples(items) => {
                for item in items.iter() {
                    theme::card(p).show(ui, |ui| {
                        ui.label(RichText::new(item.ko).size(15.0).strong().color(p.text));
                        if !item.rom.is_empty() {
                            ui.label(RichText::new(item.rom).size(11.0).italics().color(p.accent));
                        }
                        ui.label(RichText::new(item.id).size(12.0).color(p.text_dim));
                    });
                    ui.add_space(6.0);
                }
            }
        }
    }

    fn quiz(&mut self, ui: &mut Ui) {
        let p = self.p();

        // Baca semua yang dibutuhkan lebih dulu agar pinjaman `self.session`
        // berakhir sebelum `self` diubah.
        let Some(session) = self.session.as_ref() else {
            ui.label("Tidak ada kuis aktif.");
            if ui.button("Kembali").clicked() {
                self.go(Screen::Home);
            }
            return;
        };
        let total = session.total();
        let index = session.index;
        let correct = session.correct;
        let percent = session.percent();
        let current = session.current().cloned();
        let topic_id = session.topic_id;
        let answered = session.correct + session.wrong;

        if current.is_none() {
            // Sesi selesai: rekam lalu pindah ke layar hasil.
            self.progress
                .record(topic_id, correct as u32, answered as u32);
            self.persist();
            self.go(Screen::Result);
            return;
        }
        let question = current.expect("sudah diperiksa");

        ui.label(
            RichText::new(format!(
                "Soal {}/{} · benar {}",
                (index + 1).min(total),
                total,
                correct
            ))
            .size(11.0)
            .color(p.text_dim),
        );
        ui.add(
            egui::ProgressBar::new(percent as f32 / 100.0)
                .desired_width(f32::INFINITY)
                .fill(p.accent),
        );
        ui.add_space(10.0);

        theme::card(p).show(ui, |ui| {
            ui.label(
                RichText::new(question.text)
                    .size(16.0)
                    .strong()
                    .color(p.text),
            );
        });
        ui.add_space(8.0);

        let locked = self.feedback.is_some();
        let mut chosen: Option<usize> = None;

        for (i, option) in question.options.iter().enumerate() {
            let fill = match &self.feedback {
                Some(fb) if i == fb.correct => p.ok.gamma_multiply(0.35),
                Some(fb) if i == fb.chosen => p.bad.gamma_multiply(0.35),
                Some(_) => p.surface_alt,
                None => p.surface_alt,
            };

            let label = format!("{}   {}", Question::letter(i), option);
            let button = Button::new(RichText::new(label).size(14.0))
                .min_size(egui::vec2(ui.available_width(), 46.0))
                .fill(fill);
            if ui.add_enabled(!locked, button).clicked() {
                chosen = Some(i);
            }
            ui.add_space(5.0);
        }

        if let Some(choice) = chosen {
            let answer = question.answer;
            if let Some(session) = self.session.as_mut() {
                if let Some(verdict) = session.answer(choice) {
                    self.feedback = Some(Feedback {
                        verdict,
                        chosen: choice,
                        correct: answer,
                        why: question.why,
                    });
                }
            }
        }

        if let Some(fb) = &self.feedback {
            ui.add_space(6.0);
            theme::card_alt(p).show(ui, |ui| {
                let (mark, color) = match fb.verdict {
                    Verdict::Correct => ("✓ Benar", p.ok),
                    Verdict::Wrong => ("✗ Salah", p.bad),
                };
                ui.label(RichText::new(mark).size(14.0).strong().color(color));
                if fb.verdict == Verdict::Wrong {
                    ui.label(
                        RichText::new(format!(
                            "Jawaban benar: {}",
                            question.options.get(fb.correct).copied().unwrap_or("—")
                        ))
                        .size(13.0)
                        .color(p.text),
                    );
                }
                ui.label(RichText::new(fb.why).size(12.0).color(p.text_dim));
            });
            ui.add_space(8.0);

            let last = index + 1 >= total;
            let text = if last {
                "Lihat hasil"
            } else {
                "Soal berikutnya →"
            };
            if ui
                .add(
                    Button::new(RichText::new(text).size(15.0))
                        .min_size(egui::vec2(ui.available_width(), 48.0))
                        .fill(p.accent),
                )
                .clicked()
            {
                self.feedback = None;
            }
        } else if ui
            .add(
                Button::new(RichText::new("Keluar dari kuis").size(13.0))
                    .min_size(egui::vec2(ui.available_width(), 42.0)),
            )
            .clicked()
        {
            self.session = None;
            self.go(Screen::Home);
        }

        ui.add_space(10.0);
    }

    fn result(&mut self, ui: &mut Ui) {
        let p = self.p();

        let Some(session) = self.session.as_ref() else {
            self.go(Screen::Home);
            return;
        };
        let percent = session.percent();
        let correct = session.correct;
        let wrong = session.wrong;
        let grade = session.grade();
        let title = session.topic_title.clone();
        let topic_id = session.topic_id;
        let mistakes: Vec<(String, String, String)> = session
            .mistakes
            .iter()
            .map(|m| (m.question.clone(), m.chosen.clone(), m.correct.clone()))
            .collect();

        theme::card(p).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!("{percent}%"))
                        .size(42.0)
                        .strong()
                        .color(p.accent),
                );
                ui.label(RichText::new(&title).size(14.0).color(p.text_dim));
                ui.add_space(4.0);
                ui.label(RichText::new(grade).size(14.0).color(p.text));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("benar {correct} · salah {wrong}"))
                        .size(12.0)
                        .color(p.text_dim),
                );
            });
        });
        ui.add_space(10.0);

        if !mistakes.is_empty() {
            ui.label(
                RichText::new("Yang perlu diulang")
                    .size(13.0)
                    .strong()
                    .color(p.text_dim),
            );
            ui.add_space(5.0);
            for (i, (question, chosen, correct)) in mistakes.iter().enumerate() {
                theme::card_alt(p).show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{}. {question}", i + 1))
                            .size(13.0)
                            .color(p.text),
                    );
                    ui.label(
                        RichText::new(format!("jawabanmu: {chosen}"))
                            .size(12.0)
                            .color(p.bad),
                    );
                    ui.label(
                        RichText::new(format!("seharusnya: {correct}"))
                            .size(12.0)
                            .color(p.ok),
                    );
                });
                ui.add_space(5.0);
            }
        }

        ui.add_space(8.0);
        let width = (ui.available_width() - 10.0) / 2.0;
        let mut retry = false;
        let mut home = false;
        ui.horizontal(|ui| {
            if ui
                .add(
                    Button::new(RichText::new("Ulangi").size(15.0))
                        .min_size(egui::vec2(width, 48.0))
                        .fill(p.accent),
                )
                .clicked()
            {
                retry = true;
            }
            if ui
                .add(
                    Button::new(RichText::new("Daftar topik").size(15.0))
                        .min_size(egui::vec2(width, 48.0)),
                )
                .clicked()
            {
                home = true;
            }
        });

        if retry {
            if let Some(topic) = data::find(topic_id) {
                self.start_quiz(topic, 0);
            }
        } else if home {
            self.session = None;
            self.go(Screen::Home);
        }
        ui.add_space(10.0);
    }

    fn progress_screen(&mut self, ui: &mut Ui) {
        let p = self.p();
        let done = self.progress.completed_topic_count();
        let total = data::TOPICS.len();
        let average = self.progress.average();

        theme::card(p).show(ui, |ui| {
            ui.label(
                RichText::new("Progres belajar")
                    .size(15.0)
                    .strong()
                    .color(p.text),
            );
            ui.label(
                RichText::new(format!("{done} dari {total} topik dikerjakan"))
                    .size(12.0)
                    .color(p.text_dim),
            );
            ui.add_space(6.0);
            let fraction = if total == 0 {
                0.0
            } else {
                done as f32 / total as f32
            };
            ui.add(
                egui::ProgressBar::new(fraction)
                    .desired_width(f32::INFINITY)
                    .fill(p.ok)
                    .text(RichText::new(format!("{done}/{total}")).size(11.0)),
            );
            if let Some(avg) = average {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("Rata-rata skor terbaik: {avg}%"))
                        .size(12.0)
                        .color(p.text_dim),
                );
            }
        });
        ui.add_space(9.0);

        for topic in data::TOPICS {
            let stat = self.progress.get(topic.id).cloned();
            theme::card(p).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(topic.title).size(14.0).strong().color(p.text));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| match &stat {
                        Some(s) => {
                            ui.label(
                                RichText::new(format!("{}%", s.best))
                                    .size(13.0)
                                    .strong()
                                    .color(p.ok),
                            );
                        }
                        None => {
                            ui.label(RichText::new("belum").size(11.0).color(p.text_dim));
                        }
                    });
                });
                if let Some(s) = &stat {
                    ui.add(
                        egui::ProgressBar::new(s.best as f32 / 100.0)
                            .desired_width(f32::INFINITY)
                            .fill(p.ok),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}× dikerjakan · {}",
                            s.attempts,
                            format_timestamp(s.last_at)
                        ))
                        .size(11.0)
                        .color(p.text_dim),
                    );
                }
            });
            ui.add_space(7.0);
        }
    }
}

/* -------------------------------------------------------------------- uji */

#[cfg(test)]
mod tests {
    use super::*;

    /// `Progress` membaca variabel lingkungan global, jadi uji yang
    /// menyentuhnya harus berjalan bergantian.
    static DATA_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Arahkan progres ke folder sementara di dalam `target/`.
    fn siapkan_data(nama: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = DATA_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::path::Path::new("target").join("uji-data").join(nama);
        let _ = std::fs::remove_dir_all(&dir);
        std::env::set_var("KG_DATA_DIR", &dir);
        guard
    }

    fn gambar(app: &mut App, ctx: &egui::Context) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(430.0, 880.0),
            )),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| app.draw(ui));
        // `egui` menolak dijatuhkan bila delta tekstur belum dipakai; di sini
        // tidak ada GPU, jadi cukup dibuang secara eksplisit.
        output.textures_delta.clear();
    }

    #[test]
    fn semua_layar_dapat_digambar_tanpa_panik() {
        let _guard = siapkan_data("layar");
        let ctx = egui::Context::default();
        let mut app = App::with_context(&ctx);

        // Beranda.
        gambar(&mut app, &ctx);
        assert_eq!(app.screen, Screen::Home);

        // Setiap layar materi, satu per satu.
        for topic in data::TOPICS {
            app.go(Screen::Topic(topic.id));
            gambar(&mut app, &ctx);
            assert_eq!(app.screen, Screen::Topic(topic.id));
        }

        // Layar progres.
        app.go(Screen::Progress);
        gambar(&mut app, &ctx);
        assert_eq!(app.screen, Screen::Progress);
    }

    #[test]
    fn kuis_menjawab_benar_semua_merekam_progres() {
        let _guard = siapkan_data("kuis-benar");
        let ctx = egui::Context::default();
        let mut app = App::with_context(&ctx);

        let topic = data::find("eun-neun").expect("topik contoh ada");
        app.start_quiz(topic, 0);
        assert_eq!(app.screen, Screen::Quiz);
        gambar(&mut app, &ctx);

        // Jawab semua soal dengan kunci yang benar.
        let mut dijawab = 0;
        while let Some(session) = app.session.as_mut() {
            let Some(question) = session.current() else {
                break;
            };
            let kunci = question.answer;
            let hasil = session.answer(kunci);
            assert!(hasil.is_some(), "jawaban pertama harus dinilai");
            dijawab += 1;
        }
        assert!(dijawab > 0, "kuis harus punya soal");

        // Bingkai berikutnya melihat sesi kosong lalu pindah ke layar hasil.
        gambar(&mut app, &ctx);
        assert_eq!(app.screen, Screen::Result);

        // Hasilnya tersimpan di disk dan terbaca lagi.
        let tersimpan = Progress::load();
        let stat = tersimpan.get(topic.id).expect("statistik topik tersimpan");
        assert_eq!(stat.best, 100);
        assert_eq!(stat.attempts, 1);
        assert!(
            korean_rust_grammar::progress::progress_path().exists(),
            "berkas progres harus tertulis"
        );
    }

    #[test]
    fn jawaban_salah_muncul_sebagai_umpan_balik() {
        let _guard = siapkan_data("kuis-salah");
        let ctx = egui::Context::default();
        let mut app = App::with_context(&ctx);

        let topic = data::find("eun-neun").expect("topik contoh ada");
        app.start_quiz(topic, 0);

        let (salah, kunci) = {
            let session = app.session.as_ref().expect("sesi aktif");
            let question = session.current().expect("soal pertama");
            // Pilih opsi pertama yang bukan kunci.
            let salah = (0..question.options.len())
                .find(|i| *i != question.answer)
                .expect("ada opsi pengecoh");
            (salah, question.answer)
        };

        let verdict = app
            .session
            .as_mut()
            .and_then(|s| s.answer(salah))
            .expect("jawaban dinilai");
        assert_eq!(verdict, Verdict::Wrong);

        app.feedback = Some(Feedback {
            verdict,
            chosen: salah,
            correct: kunci,
            why: "uji",
        });
        gambar(&mut app, &ctx);
        assert_eq!(app.screen, Screen::Quiz);
    }

    #[test]
    fn tema_dapat_diganti_bolak_balik() {
        let _guard = siapkan_data("tema");
        let ctx = egui::Context::default();
        let mut app = App::with_context(&ctx);

        let awal = app.mode;
        app.mode = app.mode.toggled();
        gambar(&mut app, &ctx);
        assert_ne!(app.mode, awal);
        assert_eq!(app.applied_mode, Some(app.mode));
    }
}

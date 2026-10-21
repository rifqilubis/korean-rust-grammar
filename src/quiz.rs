//! Mesin kuis pilihan ganda.
//!
//! Modul ini bebas dari I/O: ia hanya menyiapkan daftar soal (opsi sudah
//! diacak), menilai jawaban, dan menyimpan rekap kesalahan. Dengan begitu
//! seluruh logika bisa diuji tanpa terminal.

use crate::model::{QuizItem, Topic};
use crate::rng::Rng;

/// Satu soal yang sudah siap ditampilkan (opsi sudah diacak).
#[derive(Clone, Debug)]
pub struct Question {
    /// Nomor soal dalam sesi (mulai dari 1).
    pub number: usize,
    /// Teks pertanyaan.
    pub text: &'static str,
    /// Empat pilihan yang sudah diacak.
    pub options: [&'static str; 4],
    /// Indeks jawaban benar pada `options`.
    pub answer: usize,
    /// Penjelasan jawaban.
    pub why: &'static str,
}

impl Question {
    /// Teks jawaban benar.
    pub fn correct_text(&self) -> &'static str {
        self.options[self.answer]
    }

    /// Huruf pilihan: A, B, C, D.
    pub fn letter(index: usize) -> char {
        const LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];
        LETTERS.get(index).copied().unwrap_or('?')
    }
}

/// Hasil satu jawaban.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Correct,
    Wrong,
}

/// Rekap satu kesalahan untuk ditampilkan di akhir sesi.
#[derive(Clone, Debug)]
pub struct Mistake {
    pub question: String,
    pub chosen: String,
    pub correct: String,
    pub why: &'static str,
}

/// Susun ulang satu soal: opsi diacak, indeks jawaban ikut disesuaikan.
fn prepare(item: &QuizItem, number: usize, rng: &mut Rng) -> Question {
    let mut order: [usize; 4] = [0, 1, 2, 3];
    rng.shuffle(&mut order);

    let mut options = ["", "", "", ""];
    let mut answer = 0;
    for (slot, original) in order.iter().enumerate() {
        options[slot] = item.options[*original];
        if *original == item.answer {
            answer = slot;
        }
    }

    Question {
        number,
        text: item.q,
        options,
        answer,
        why: item.why,
    }
}

/// Sesi kuis: daftar soal + akumulasi skor.
#[derive(Clone, Debug)]
pub struct Session {
    pub topic_id: &'static str,
    pub topic_no: u32,
    pub topic_title: String,
    pub questions: Vec<Question>,
    pub index: usize,
    pub correct: usize,
    pub wrong: usize,
    pub mistakes: Vec<Mistake>,
}

impl Session {
    /// Bangun sesi baru. `limit == 0` berarti semua soal.
    pub fn new(topic: &'static Topic, limit: usize, rng: &mut Rng) -> Session {
        let mut pool: Vec<&QuizItem> = topic.quiz.iter().collect();
        rng.shuffle(&mut pool);
        if limit > 0 && limit < pool.len() {
            pool.truncate(limit);
        }

        let questions: Vec<Question> = pool
            .iter()
            .enumerate()
            .map(|(i, item)| prepare(item, i + 1, rng))
            .collect();

        Session {
            topic_id: topic.id,
            topic_no: topic.no,
            topic_title: topic.title.to_string(),
            questions,
            index: 0,
            correct: 0,
            wrong: 0,
            mistakes: Vec::new(),
        }
    }

    /// Soal yang sedang dikerjakan, bila ada.
    pub fn current(&self) -> Option<&Question> {
        self.questions.get(self.index)
    }

    pub fn is_finished(&self) -> bool {
        self.index >= self.questions.len()
    }

    pub fn total(&self) -> usize {
        self.questions.len()
    }

    /// Nilai jawaban (indeks 0..=3). Mengembalikan `None` bila sesi selesai.
    pub fn answer(&mut self, choice: usize) -> Option<Verdict> {
        let question = self.questions.get(self.index)?;
        let verdict = if choice == question.answer {
            self.correct += 1;
            Verdict::Correct
        } else {
            self.wrong += 1;
            self.mistakes.push(Mistake {
                question: question.text.to_string(),
                chosen: question
                    .options
                    .get(choice)
                    .copied()
                    .unwrap_or("(tidak dijawab)")
                    .to_string(),
                correct: question.correct_text().to_string(),
                why: question.why,
            });
            Verdict::Wrong
        };
        self.index += 1;
        Some(verdict)
    }

    /// Persentase benar (0..=100).
    pub fn percent(&self) -> u32 {
        let answered = self.correct + self.wrong;
        if answered == 0 {
            0
        } else {
            ((self.correct * 100) as f64 / answered as f64).round() as u32
        }
    }

    /// Predikat singkat berdasarkan persentase.
    pub fn grade(&self) -> &'static str {
        match self.percent() {
            90..=100 => "Luar biasa! 완벽해요!",
            75..=89 => "Bagus! 잘했어요!",
            60..=74 => "Cukup — ulangi bagian yang salah.",
            _ => "Perlu belajar lagi. 화이팅!",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn session_for(id: &str, limit: usize) -> Session {
        let topic = data::find(id).expect("topik ada");
        Session::new(topic, limit, &mut Rng::new(1))
    }

    #[test]
    fn session_respects_limit() {
        let s = session_for("hangeul", 2);
        assert_eq!(s.total(), 2);
        assert_eq!(s.questions.len(), 2);
    }

    #[test]
    fn limit_larger_than_bank_is_safe() {
        let topic = data::find("hangeul").unwrap();
        let s = session_for("hangeul", topic.quiz.len() + 10);
        assert_eq!(s.total(), topic.quiz.len());
    }

    #[test]
    fn options_preserve_all_answers() {
        let topic = data::find("eun-neun").unwrap();
        let s = session_for("eun-neun", 0);
        for q in s.questions.iter() {
            // Cocokkan lewat teks soal karena urutan soal sudah diacak.
            let original = topic
                .quiz
                .iter()
                .find(|item| item.q == q.text)
                .expect("soal harus berasal dari bank");
            let mut sorted = q.options;
            sorted.sort_unstable();
            let mut expect = original.options;
            expect.sort_unstable();
            assert_eq!(sorted, expect, "opsi tidak boleh hilang/berubah");
            assert_eq!(q.options[q.answer], original.options[original.answer]);
        }
    }

    #[test]
    fn answering_all_correct_gives_hundred() {
        let mut s = session_for("hangeul", 0);
        while let Some(q) = s.current() {
            let answer = q.answer;
            assert_eq!(s.answer(answer), Some(Verdict::Correct));
        }
        assert!(s.is_finished());
        assert_eq!(s.wrong, 0);
        assert_eq!(s.percent(), 100);
        assert!(s.mistakes.is_empty());
    }

    #[test]
    fn wrong_answer_is_recorded() {
        let mut s = session_for("eun-neun", 1);
        let answer = s.current().unwrap().answer;
        let wrong_choice = (answer + 1) % 4;
        assert_eq!(s.answer(wrong_choice), Some(Verdict::Wrong));
        assert_eq!(s.mistakes.len(), 1);
        assert_eq!(s.percent(), 0);
        assert!(!s.mistakes[0].correct.is_empty());
    }

    #[test]
    fn answering_past_end_returns_none() {
        let mut s = session_for("urutan", 1);
        let answer = s.current().unwrap().answer;
        s.answer(answer);
        assert_eq!(s.answer(0), None);
    }

    #[test]
    fn grade_thresholds() {
        let mut s = session_for("hangeul", 0);
        assert_eq!(s.percent(), 0);
        s.correct = 9;
        s.wrong = 1;
        assert_eq!(s.percent(), 90);
        assert!(s.grade().contains("Luar biasa"));
    }

    #[test]
    fn letters_are_abcd() {
        assert_eq!(Question::letter(0), 'A');
        assert_eq!(Question::letter(3), 'D');
        assert_eq!(Question::letter(9), '?');
    }
}

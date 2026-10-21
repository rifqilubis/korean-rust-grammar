//! Titik masuk biner `kg`.
//!
//! Biner ini hanya membungkus [`korean_rust_grammar::cli::run`] dengan I/O
//! standar proses. Semua logika ada di pustaka agar bisa diuji.

use std::io;
use std::process::ExitCode;

use korean_rust_grammar::cli::{self, Outcome};
use korean_rust_grammar::ui;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = io::BufWriter::new(stdout.lock());

    let outcome = cli::run(&args, &mut input, &mut output);

    if let Err(e) = io::Write::flush(&mut output) {
        // Pipa tertutup (mis. `kg list | head`) bukan alasan untuk gagal keras.
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("gagal menulis output: {}", e);
            return ExitCode::from(1);
        }
    }

    match outcome {
        Outcome::Ok => ExitCode::SUCCESS,
        Outcome::UserError(message) => {
            let color = ui::color_enabled();
            eprintln!(
                "{} {}",
                ui::paint("kesalahan:", ui::Color::Red, color),
                message
            );
            ExitCode::from(2)
        }
    }
}

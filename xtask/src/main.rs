use conformance::ConformanceTestSuite;
use lexer::LexerTestSuite;
use suite::TestSuite;

use std::io::Write;

mod compiler;
mod conformance;
mod lexer;
mod suite;
mod utils;
fn main() {
    // std::panic::set_hook(Box::new(|_info| {}));
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
    let mut out = std::io::stdout();

    let lexer_summary = LexerTestSuite::new().run();
    let conformance_summary = ConformanceTestSuite::new().run();
    let compiler_summary = compiler::CompilerTestSuite::new().run();

    writeln!(out, "---------- Summary(Lexer) ----------\n").expect("Unable to write summary");
    lexer_summary.show_and_write_summary(&mut out);
    conformance_summary.show_and_write_summary(&mut out);
    compiler_summary.show_and_write_summary(&mut out);
}

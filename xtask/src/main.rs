use conformance::ConformanceTestSuite;
use lexer::LexerTestSuite;
use suite::TestSuite;

use std::io::Write;

mod conformance;
mod lexer;
mod suite;
fn main() {
    // std::panic::set_hook(Box::new(|_info| {}));
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
    let mut out = std::io::stdout();

    let lexer_summary = LexerTestSuite::new().run();
    let conformance_summary = ConformanceTestSuite::new().run();

    writeln!(out, "---------- Summary ----------\n").expect("Unable to write summary");
    lexer_summary.show_and_write_summary(&mut out);
    conformance_summary.show_and_write_summary(&mut out);
}

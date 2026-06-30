use noctavox::addons::parse_args;
use std::io::Write;

fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1"); }
    let _ = write!(std::io::stdout(), "\x1b]0;NoctaVox\x07");
    let _ = std::io::stdout().flush();

    if std::env::args().len() == 1 {
        match noctavox::app_core::NoctaVox::new() {
            Ok(mut app) => app.run(),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1)
            }
        }
        return;
    }

    parse_args();
}

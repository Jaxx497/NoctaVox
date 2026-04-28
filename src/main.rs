fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    match noctavox::app_core::NoctaVox::new() {
        Ok(mut app) => app.run(),
        Err(e) => eprintln!("{e}"),
    };
}

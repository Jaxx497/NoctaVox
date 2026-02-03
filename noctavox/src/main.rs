fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    noctavox::app_core::NoctaVox::new().run();
}

use std::{backtrace::Backtrace, panic::PanicHookInfo, thread};

fn panic_hook(panic_info: &PanicHookInfo) {
    let backtrace = Backtrace::capture().to_string();

    let thread = thread::current();
    let thread = thread.name().unwrap_or("<unknown>");
    let locataion = panic_info
        .location()
        .map_or_else(|| "<unknown>".to_string(), ToString::to_string);

    let message = panic_info
        .payload()
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<&str>()
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "<unknown>".to_string());

    tracing::error!("thread '{}' panicked at {}:", thread, locataion);
    eprintln!("{}\n", message);
    eprintln!("Backtrace:\n{}", backtrace);

    std::process::exit(1);
}

/// Initialize panic handling system
/// Sets up panic handler and starts dedicated panic monitoring thread
pub fn init_panic_handling() {
    // Ensure backtrace is enabled for panics
    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "full");
            std::env::set_var("RUST_LIB_BACKTRACE", "1");
        }
    }

    std::panic::set_hook(Box::new(panic_hook));
}

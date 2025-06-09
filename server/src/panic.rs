use smol::channel::{Sender, unbounded};
use std::process::exit;
use tracing::error;

#[derive(Debug, Clone)]
pub struct PanicDetails {
    pub thread_name: Option<String>,
    pub message: Option<String>,
    pub location: Option<PanicDetailsLocation>,
    pub backtrace: String,
}

#[derive(Clone, Debug)]
pub struct PanicDetailsLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl std::fmt::Display for PanicDetailsLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl From<&std::panic::Location<'_>> for PanicDetailsLocation {
    fn from(location: &std::panic::Location<'_>) -> Self {
        PanicDetailsLocation {
            file: location.file().to_string(),
            line: location.line(),
            column: location.column(),
        }
    }
}

impl std::fmt::Display for PanicDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_thread_name = match &self.thread_name {
            Some(name) => format!("'{}'", name),
            None => "<unknown>".to_string(),
        };

        let fmt_message = match &self.message {
            Some(message) => format!("'{}'", message),
            None => "<no message>".to_string(),
        };

        let fmt_location = match &self.location {
            Some(location) => location.to_string(),
            None => "<no location>".to_string(),
        };

        // Apply syntax highlighting to backtrace
        let highlighted_backtrace = self.highlight_backtrace();

        write!(
            f,
            "thread {} panicked at {}, {}\n{}",
            fmt_thread_name, fmt_message, fmt_location, highlighted_backtrace
        )
    }
}

impl PanicDetails {
    /// Highlights backtrace lines that contain server or control-core crate references
    /// and the line immediately following them
    fn highlight_backtrace(&self) -> String {
        use regex::Regex;

        // Regex to match lines starting with digits followed by colon
        let line_regex = Regex::new(r"^\s*(\d+):\s*(.*)$").unwrap();

        let lines: Vec<&str> = self.backtrace.lines().collect();
        let mut result = Vec::new();
        let mut highlight_next = false;

        for (_, line) in lines.iter().enumerate() {
            if let Some(captures) = line_regex.captures(line) {
                let line_num = &captures[1];
                let content = &captures[2];

                // Check if the line contains server or control-core
                if content.contains("server") || content.contains("control-core") {
                    // Red highlighting for relevant lines
                    result.push(format!("\x1b[91m{}: {}\x1b[0m", line_num, content));
                    highlight_next = true;
                } else {
                    // Dim gray for other numbered lines
                    result.push(format!("\x1b[90m{}: {}\x1b[0m", line_num, content));
                }
            } else {
                // Non-numbered lines (headers, location info, etc.)
                if highlight_next {
                    // Highlight the line after a matched regex line in yellow
                    result.push(format!("\x1b[93m{}\x1b[0m", line));
                    highlight_next = false;
                } else {
                    // Default color for other non-numbered lines
                    result.push(line.to_string());
                }
            }
        }

        result.join("\n")
    }
}

/// Thread-level panic handler for general thread crashes
/// Sends detailed panic information including backtrace
pub fn send_panic(thread_panic_tx: Sender<PanicDetails>) {
    // Ensure backtrace is enabled for panics
    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "full");
        }
    }

    std::panic::set_hook(Box::new(move |panic_info| {
        // Capture backtrace at panic site
        let backtrace = std::backtrace::Backtrace::force_capture();

        let thread_name = std::thread::current().name().map(|name| name.to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            Some(s.to_string())
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            Some(s.clone())
        } else {
            None
        };

        let panic_details = PanicDetails {
            thread_name: thread_name,
            message: message,
            location: panic_info.location().map(|loc| loc.into()),
            backtrace: backtrace.to_string(),
        };

        // Send detailed panic info through channel
        let _ = smol::block_on(thread_panic_tx.send(panic_details));
    }));
}

/// Initialize panic handling system
/// Sets up panic handler and starts dedicated panic monitoring thread
pub fn init_panic() -> Sender<PanicDetails> {
    let (thread_panic_tx, thread_panic_rx) = unbounded::<PanicDetails>();
    send_panic(thread_panic_tx.clone());

    // Start panic monitoring thread
    let thread_panic_rx_clone = thread_panic_rx.clone();
    std::thread::Builder::new()
        .name("panic".to_string())
        .spawn(move || {
            // Create an executor for this thread
            let rt = smol::Executor::new();
            smol::block_on(rt.run(async {
                loop {
                    match thread_panic_rx_clone.recv().await {
                        Ok(panic_details) => {
                            error!("{}", panic_details);
                            exit(1);
                        }
                        Err(_) => {
                            // Channel closed, exit gracefully
                            break;
                        }
                    }
                }
            }))
        })
        .expect("Failed to spawn panic monitoring thread");

    thread_panic_tx
}

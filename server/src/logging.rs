use tracing_subscriber::EnvFilter;

use crate::config::LogLevel;

/// Plain-text logs on stdout, which the Supervisor shows in the app's Log tab.
/// `RUST_LOG` wins over the configured level when set.
pub fn init(level: LogLevel) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = level.as_directive();
        EnvFilter::new(format!("warn,memory_card={level},tower_http={level}"))
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(std::io::stdout)
        .init();
}

pub mod log {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    /// Initialise global tracing subscriber with env filter.
    pub fn init() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }
} 
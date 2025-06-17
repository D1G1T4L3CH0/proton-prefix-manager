use env_logger::{Builder, Env};

/// Initialize logging with optional debug output.
pub fn init(debug: bool) {
    let env = Env::default().default_filter_or(if debug { "debug" } else { "info" });
    Builder::from_env(env).init();
}

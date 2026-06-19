mod settings;

pub use settings::Settings;

/// Backward-compatible alias used across the codebase and tests.
pub type Config = Settings;

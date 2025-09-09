pub mod creation;
pub mod defaults;
pub mod discovery;
pub mod future_example;
pub mod loader;
pub mod path_expansion;
pub mod paths;
pub mod schema;

// Re-exports for convenience
pub use loader::{load_config, load_config_silent};
pub use paths::{cache_dir, data_dir, expand_optional_path, expand_required_path};
pub use schema::Config;

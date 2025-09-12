// Adapter manifest definitions

use serde::{Deserialize, Serialize};

/// Adapter manifest describing essential metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterManifest {
    pub name: String,
    pub version: String,
    /// Optional: Human-readable display name (frontend decides fallback to name)
    pub display_name: Option<String>,
    /// Optional: Available models (if undefined, UI will show text field)
    pub models: Option<Vec<String>>,
}

impl AdapterManifest {
    /// Create a default manifest from config values
    pub fn default_from_config(provider: &str, version: &str) -> Self {
        Self {
            name: provider.to_string(),
            version: version.to_string(),
            display_name: None, // Frontend decides fallback behavior
            models: None,       // Default: UI shows text field
        }
    }
}

/// Common imports for ai_messenger applications.
///
/// This prelude brings in the most commonly used types and functions
/// when working with ai_messenger as a library.
///
/// # Example
///
/// ```rust,no_run
/// use ai_messenger::prelude::*;
///
/// // Now you have access to:
/// // - Config, ServerConfig
/// // - Error, Result
/// // - init, init_with_logging
/// // - tracing macros (debug, info, warn, error, trace)
///
/// let config = Config::default();
/// info!("Starting with config: {:?}", config);
/// ```
// Core configuration types
pub use crate::config::schema::{Config, ServerConfig};

// Error handling
pub use crate::library::error::{Error, Result};

// Initialization functions
pub use crate::library::init::{init, init_with_logging};

// Re-export tracing for convenience when building on top of ai_messenger
pub use tracing::{debug, error, info, trace, warn};

// TODO: Add more as we implement core types
// pub use crate::library::types::{Message, Conversation, Sender, Recipient};
// pub use crate::library::types::{AIAdapter, StorageAdapter, CryptoAdapter};

/// Library initialization functions for ai_messenger.
use crate::library::error::Result;

/// Initialize ai_messenger library without touching global logging state.
///
/// This is the preferred way to initialize when ai_messenger is used as a library
/// within another application. The host application retains control over logging
/// configuration.
///
/// # Example
///
/// ```rust,no_run
/// use ai_messenger::prelude::*;
///
/// // Host app controls logging
/// tracing_subscriber::fmt::init();
///
/// // Initialize ai_messenger (no logging changes)
/// ai_messenger::init()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn init() -> Result<()> {
    // NOTE: Libraries should NOT initialize global tracing subscriber!
    // That's the job of the host application.
    //
    // If the host app hasn't set up tracing, our tracing calls will simply be no-ops.
    // If they have, our tracing calls will work seamlessly.

    // TODO: Any other global initialization needed when we implement server
    // - WASM runtime setup
    // - Adapter registry initialization
    Ok(())
}

/// Initialize ai_messenger with logging for standalone usage.
///
/// This is a convenience function for applications that use ai_messenger as their
/// main component and want ai_messenger to handle logging setup.
///
/// This is separate from init() because libraries shouldn't control global state
/// unless explicitly requested.
///
/// # Example
///
/// ```rust,no_run
/// use ai_messenger::prelude::*;
///
/// // ai_messenger controls logging
/// ai_messenger::init_with_logging("debug")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn init_with_logging(level: &str) -> Result<()> {
    // This is for apps that embed ai_messenger as their main component
    crate::utils::logger::init_logging(level)?;
    init()
}

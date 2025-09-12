use super::router;
use crate::adapter::services::AdapterRegistry;
use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server startup configuration
#[derive(Debug)]
pub struct ServerStartupConfig {
    pub config: Config,
    pub config_dir: Option<PathBuf>,
    pub host: String,
    pub log_level: String,
    pub port: u16,
}

/// Start the server with the given configuration
pub async fn start(startup_config: ServerStartupConfig) -> Result<()> {
    let base_path = &startup_config.config.server.base_path;

    // Initialize adapter registry with configuration
    let mut adapter_registry = AdapterRegistry::new().await?;

    // Use data directory from config with fallback to defaults
    let data_dir = if let Some(data_dir) = &startup_config.config.storage.data_dir {
        crate::config::paths::expand_required_path(data_dir, startup_config.config_dir.as_deref())
    } else {
        // Use default data directory
        crate::config::defaults::default_data_dir()
    };

    tracing::info!(
        "Initializing adapters from config, data_dir: {:?}",
        data_dir
    );
    adapter_registry
        .initialize_from_config(&startup_config.config, &data_dir)
        .await?;

    tracing::info!("Adapters initialized successfully");

    // Wrap registry in Arc<RwLock> for sharing across routes
    let shared_registry = Arc::new(RwLock::new(adapter_registry));

    // Build the router with adapter registry
    let app = router::build_router(base_path, shared_registry);

    // Create listener
    let addr = format!("{}:{}", startup_config.host, startup_config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Show startup messages based on log level
    show_startup_messages(&startup_config, &addr, base_path);

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Display startup messages based on log level
fn show_startup_messages(startup_config: &ServerStartupConfig, addr: &str, base_path: &str) {
    match startup_config.log_level.as_str() {
        "silent" => {
            // Only show critical startup info
        }
        "error" | "warn" | "info" => {
            print_basic_startup_info(addr, base_path);
        }
        "debug" => {
            print_debug_startup_info(startup_config, addr, base_path);
            print_basic_startup_info(addr, base_path);
        }
        _ => {
            // Fallback to info level
            println!("Server running on http://{}", addr);
        }
    }
}

/// Print basic server startup information
fn print_basic_startup_info(addr: &str, base_path: &str) {
    println!("Server running on http://{}", addr);
    print_api_endpoints(addr, base_path);
}

/// Print debug-specific startup information
fn print_debug_startup_info(startup_config: &ServerStartupConfig, addr: &str, base_path: &str) {
    println!(
        "Starting server on {} (base_path: '{}')",
        addr,
        if base_path.is_empty() { "/" } else { base_path }
    );

    if let Some(config_dir) = &startup_config.config_dir {
        println!("Config directory: {}", config_dir.display());
    } else {
        println!("Config directory: <none> (using defaults)");
    }
}

/// Print API endpoint information
fn print_api_endpoints(addr: &str, base_path: &str) {
    if !base_path.is_empty() {
        println!(
            "API endpoints available at: http://{}/{}/v1/*",
            addr, base_path
        );
    } else {
        println!("API endpoints available at: http://{}/v1/*", addr);
    }
}

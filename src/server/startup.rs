use super::router;
use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

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

    // Build the router
    let app = router::build_router(base_path);

    // Create listener
    let addr = format!("{}:{}", startup_config.host, startup_config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Logging based on log level
    match startup_config.log_level.as_str() {
        "silent" => {
            // Only show critical startup info
        }
        "error" | "warn" | "info" => {
            println!("Server running on http://{}", addr);
            if !base_path.is_empty() {
                println!(
                    "API endpoints available at: http://{}/{}/v1/*",
                    addr, base_path
                );
            } else {
                println!("API endpoints available at: http://{}/v1/*", addr);
            }
        }
        "debug" => {
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
            println!("Server running on http://{}", addr);
            if !base_path.is_empty() {
                println!(
                    "API endpoints available at: http://{}/{}/v1/*",
                    addr, base_path
                );
            } else {
                println!("API endpoints available at: http://{}/v1/*", addr);
            }
        }
        _ => {
            // Fallback to info level
            println!("Server running on http://{}", addr);
        }
    }

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

use anyhow::Result;

mod adapter;
mod cli;
mod config;
mod routes;
mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli::build().get_matches();

    match matches.subcommand() {
        Some(("serve", sub_m)) => {
            cli::commands::serve::run(sub_m).await?;
        }
        Some(("cache", sub_m)) => {
            cli::commands::cache::run(sub_m).await?;
        }
        Some(("data", sub_m)) => {
            cli::commands::data::run(sub_m).await?;
        }
        Some(("help", sub_m)) => {
            // Handle help command
            if let Some(cmd_name) = sub_m.get_one::<String>("command") {
                // Show help for specific command
                match cmd_name.as_str() {
                    "serve" => {
                        let mut serve_cmd = cli::commands::serve::command();
                        serve_cmd.print_help()?;
                    }
                    "cache" => {
                        let mut cache_cmd = cli::commands::cache::command();
                        cache_cmd.print_help()?;
                    }
                    "data" => {
                        let mut data_cmd = cli::commands::data::command();
                        data_cmd.print_help()?;
                    }
                    "help" => {
                        let mut app = cli::build();
                        let help_cmd = app.find_subcommand_mut("help").unwrap();
                        help_cmd.print_help()?;
                    }
                    _ => {
                        eprintln!("Unknown command: {}", cmd_name);
                        std::process::exit(1);
                    }
                }
            } else {
                // Show general help
                let mut app = cli::build();
                app.print_help()?;
            }
        }
        _ => {
            // No subcommand provided, show help
            let mut app = cli::build();
            app.print_help()?;
        }
    }

    Ok(())
}

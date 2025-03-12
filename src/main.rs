use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod github;
mod tui;
mod ai;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GitHub username to look up starred repositories
    #[arg(short, long)]
    username: Option<String>,

    /// GitHub personal access token
    #[arg(short, long)]
    token: Option<String>,

    /// OpenAI API key for AI suggestions
    #[arg(long)]
    openai_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Parse command line arguments
    let args = Args::parse();
    
    // Load configuration
    let mut config = config::load_config()?;
    
    // Override config with command line arguments if provided
    if let Some(token) = args.token {
        config.github_token = Some(token);
    }
    
    if let Some(username) = args.username {
        config.github_username = Some(username);
    }
    
    if let Some(openai_key) = args.openai_key {
        config.openai_api_key = Some(openai_key);
    }

    // Check if we have required configuration
    if config.github_username.is_none() || config.github_token.is_none() {
        println!("GitHub username and token are required.");
        println!("Please provide them via command line arguments or config file.");
        return Ok(());
    }

    // Initialize application state
    let app = app::App::new(config).await?;
    
    // Run the TUI application
    tui::run(app).await?;

    Ok(())
}

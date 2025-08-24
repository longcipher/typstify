use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::Result;
use tracing::info;
use typstify_ssg::{config::AppConfig, Site};

#[derive(Parser)]
#[command(name = "typstify-ssg")]
#[command(about = "A static site generator for Markdown and Typst content")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build static site from content directory
    Build {
        /// Override content directory path
        #[arg(short, long)]
        content: Option<PathBuf>,
        /// Override output directory path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Serve the built site locally
    Serve {
        /// Override site directory to serve
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Override port to serve on
        #[arg(short, long)]
        port: Option<u16>,
    },
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load configuration
    let config_path = cli.config.as_ref().map(|p| p.to_string_lossy().to_string());
    let app_config = AppConfig::load_or_default(config_path.as_deref())?;

    info!(
        "Loaded configuration: {}",
        cli.config
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "defaults".to_string())
    );

    match cli.command.unwrap_or(Commands::Build {
        content: None,
        output: None,
    }) {
        Commands::Build { content, output } => {
            let content_dir = content.unwrap_or_else(|| app_config.build.content_dir.clone());
            let output_dir = output.unwrap_or_else(|| app_config.build.output_dir.clone());

            info!("🚀 Building Typstify site...");
            info!("   Content: {}", content_dir.display());
            info!("   Output:  {}", output_dir.display());

            // Create site and build
            let mut site = Site::new(content_dir, output_dir).with_app_config(app_config);
            site.scan_content()?;
            site.init_search_engine()?;
            site.build()?;

            info!("✅ Site built successfully!");
        }
        Commands::Serve { dir, port } => {
            let serve_dir = dir.unwrap_or_else(|| app_config.build.output_dir.clone());
            let serve_port = port.unwrap_or(app_config.dev.port);

            info!(
                "🌐 Serving site from {} on port {}",
                serve_dir.display(),
                serve_port
            );
            info!("   Visit: http://localhost:{}", serve_port);

            // Simple file server implementation
            serve_directory(serve_dir, serve_port)?;
        }
    }

    Ok(())
}

fn serve_directory(dir: PathBuf, port: u16) -> Result<()> {
    use std::process::Command;

    info!("Starting HTTP server...");
    let mut cmd = Command::new("python3")
        .args(["-m", "http.server", &port.to_string()])
        .current_dir(dir)
        .spawn()?;

    // Wait for the command to finish
    let status = cmd.wait()?;

    if !status.success() {
        eyre::bail!("Server exited with error");
    }

    Ok(())
}

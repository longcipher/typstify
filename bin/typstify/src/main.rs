//! Typstify CLI
//!
//! Single binary static site generator with Typst/Markdown support.
//!
//! This is the binary entry point. The library functionality is in `lib.rs`.

use clap::Parser;
use color_eyre::eyre::Result;

/// Command-line interface for Typstify.
#[derive(Parser)]
#[command(
    name = "typstify",
    version,
    about = "A high-performance static site generator"
)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: std::path::PathBuf,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands.
#[derive(clap::Subcommand)]
enum Commands {
    /// Build the static site for production
    Build {
        /// Output directory
        #[arg(short, long, default_value = "public")]
        output: std::path::PathBuf,
        /// Include draft posts
        #[arg(long)]
        drafts: bool,
        /// Override site host (e.g., https://example.com)
        #[arg(long)]
        host: Option<String>,
        /// Override site base path (e.g., /my-blog)
        #[arg(long)]
        base_path: Option<String>,
    },
    /// Start development server with live reload
    Watch {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
    },
    /// Create new content from template
    New {
        /// Path for the new content (e.g., posts/my-article)
        path: std::path::PathBuf,
        /// Template type (post, page, typst)
        #[arg(short, long, default_value = "post")]
        template: String,
    },
    /// Validate configuration and content
    Check {
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    typstify::init_tracing(cli.verbose);

    match cli.command {
        Commands::Build {
            output,
            drafts,
            host,
            base_path,
        } => {
            typstify::cmd::build::run(
                &cli.config,
                &output,
                drafts,
                host.as_deref(),
                base_path.as_deref(),
            )?;
        }
        Commands::Watch { port, open } => {
            typstify::cmd::watch::run(&cli.config, port, open).await?;
        }
        Commands::New { path, template } => {
            typstify::cmd::new::run(&path, &template)?;
        }
        Commands::Check { strict } => {
            typstify::cmd::check::run(&cli.config, strict)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn test_cli_build_command_parsing() {
        let args = ["typstify", "build", "--output", "dist"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.config, std::path::PathBuf::from("config.toml"));
        assert_eq!(cli.verbose, 0);

        match cli.command {
            Commands::Build {
                output,
                drafts,
                host,
                base_path,
            } => {
                assert_eq!(output, std::path::PathBuf::from("dist"));
                assert!(!drafts);
                assert!(host.is_none());
                assert!(base_path.is_none());
            }
            _ => panic!("Expected Build command"),
        }
    }

    #[test]
    fn test_cli_build_with_drafts() {
        let args = ["typstify", "build", "--drafts"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Build { drafts, .. } => {
                assert!(drafts);
            }
            _ => panic!("Expected Build command"),
        }
    }

    #[test]
    fn test_cli_watch_command_parsing() {
        let args = ["typstify", "watch", "--port", "8080", "--open"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Watch { port, open } => {
                assert_eq!(port, 8080);
                assert!(open);
            }
            _ => panic!("Expected Watch command"),
        }
    }

    #[test]
    fn test_cli_new_command_parsing() {
        let args = ["typstify", "new", "posts/my-article", "--template", "typst"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::New { path, template } => {
                assert_eq!(path, std::path::PathBuf::from("posts/my-article"));
                assert_eq!(template, "typst");
            }
            _ => panic!("Expected New command"),
        }
    }

    #[test]
    fn test_cli_check_command_parsing() {
        let args = ["typstify", "check", "--strict"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Check { strict } => {
                assert!(strict);
            }
            _ => panic!("Expected Check command"),
        }
    }

    #[test]
    fn test_cli_verbosity_flags() {
        let args = ["typstify", "-vvv", "build"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn test_cli_custom_config_path() {
        let args = ["typstify", "--config", "site.toml", "build"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.config, std::path::PathBuf::from("site.toml"));
    }

    #[test]
    fn test_cli_build_with_host_and_base_path() {
        let args = [
            "typstify",
            "build",
            "--host",
            "https://example.com",
            "--base-path",
            "/blog",
        ];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Build {
                host, base_path, ..
            } => {
                assert_eq!(host.as_deref(), Some("https://example.com"));
                assert_eq!(base_path.as_deref(), Some("/blog"));
            }
            _ => panic!("Expected Build command"),
        }
    }
}

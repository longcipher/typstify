//! Watch command - development server with live reload

use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use color_eyre::eyre::{Result, WrapErr};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind};
use tokio::{net::TcpListener, sync::mpsc};
use typstify_core::Config;
use typstify_generator::Builder;

use crate::server::{LIVERELOAD_SCRIPT, ServerState, create_router};

/// Debounce interval for file changes.
const DEBOUNCE_MS: u64 = 200;

/// Run the watch command.
///
/// Starts a development server with live reload support.
pub async fn run(config_path: &Path, port: u16, open_browser: bool) -> Result<()> {
    tracing::info!(?config_path, port, "Starting watch mode");

    // Load configuration
    let mut config = Config::load(config_path).wrap_err("Failed to load configuration")?;

    // Enable drafts in development mode
    config.build.drafts = true;

    let output_dir = Path::new(&config.build.output_dir).to_path_buf();
    let content_dir_path = Path::new("content").to_path_buf();

    // Initial build
    tracing::info!("Running initial build...");
    let builder = Builder::new(config.clone(), &content_dir_path, &output_dir);
    inject_livereload_and_build(&builder, &output_dir)?;
    println!("  Initial build complete");

    // Create server state
    let state = Arc::new(ServerState::new());

    // Setup file watcher
    let (tx, mut rx) = mpsc::channel::<()>(16);
    let watcher_tx = tx.clone();

    let content_dir = Path::new("content").to_path_buf();
    let templates_dir = Path::new("templates").to_path_buf();
    let style_dir = Path::new("style").to_path_buf();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                // Only trigger on write/modify events
                if matches!(
                    event.kind,
                    EventKind::Modify(ModifyKind::Data(_))
                        | EventKind::Create(_)
                        | EventKind::Remove(_)
                ) {
                    let _ = watcher_tx.blocking_send(());
                }
            }
        },
        notify::Config::default(),
    )
    .wrap_err("Failed to create file watcher")?;

    // Watch directories
    if content_dir.exists() {
        watcher
            .watch(&content_dir, RecursiveMode::Recursive)
            .wrap_err("Failed to watch content directory")?;
        tracing::debug!("Watching content directory");
    }
    if templates_dir.exists() {
        watcher
            .watch(&templates_dir, RecursiveMode::Recursive)
            .wrap_err("Failed to watch templates directory")?;
        tracing::debug!("Watching templates directory");
    }
    if style_dir.exists() {
        watcher
            .watch(&style_dir, RecursiveMode::Recursive)
            .wrap_err("Failed to watch style directory")?;
        tracing::debug!("Watching style directory");
    }

    // Start rebuild task
    let rebuild_state = state.clone();
    let rebuild_config = config.clone();
    let rebuild_output = output_dir.clone();
    let rebuild_content = content_dir_path.clone();

    tokio::spawn(async move {
        let mut last_rebuild = Instant::now();

        while rx.recv().await.is_some() {
            // Debounce
            if last_rebuild.elapsed() < Duration::from_millis(DEBOUNCE_MS) {
                continue;
            }

            // Drain any queued events
            while rx.try_recv().is_ok() {}

            tracing::info!("File change detected, rebuilding...");
            let builder = Builder::new(rebuild_config.clone(), &rebuild_content, &rebuild_output);

            match inject_livereload_and_build(&builder, &rebuild_output) {
                Ok(_) => {
                    println!("  Rebuild complete");
                    rebuild_state.notify_reload();
                }
                Err(e) => {
                    tracing::error!("Rebuild failed: {e}");
                    eprintln!("  Rebuild failed: {e}");
                }
            }

            last_rebuild = Instant::now();
        }
    });

    // Start server
    let app = create_router(&output_dir, state);
    let addr = format!("127.0.0.1:{port}");

    let listener = TcpListener::bind(&addr)
        .await
        .wrap_err_with(|| format!("Failed to bind to {addr}"))?;

    println!();
    println!("  Dev server running at http://{addr}");
    println!("  Press Ctrl+C to stop");
    println!();

    if open_browser {
        let _ = open::that(format!("http://{addr}"));
    }

    // Keep watcher alive
    let _watcher = watcher;

    axum::serve(listener, app).await.wrap_err("Server error")?;

    Ok(())
}

/// Build and inject livereload script into HTML files.
fn inject_livereload_and_build(builder: &Builder, output_dir: &Path) -> Result<()> {
    let stats = builder.build().wrap_err("Build failed")?;

    // Inject livereload script into all HTML files
    inject_livereload_into_html(output_dir)?;

    tracing::debug!(?stats, "Build completed");
    Ok(())
}

/// Inject livereload script into all HTML files in the output directory.
fn inject_livereload_into_html(output_dir: &Path) -> Result<()> {
    use std::fs;

    for entry in walkdir::WalkDir::new(output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
    {
        let path = entry.path();
        let content = fs::read_to_string(path)?;

        // Only inject if not already present
        if !content.contains("__livereload") {
            let modified = content.replace("</body>", &format!("{LIVERELOAD_SCRIPT}</body>"));
            fs::write(path, modified)?;
        }
    }

    Ok(())
}

//! Embedded development server with live reload support

use std::{path::Path, sync::Arc, time::Duration};

use axum::{
    Router,
    response::sse::{Event, Sse},
    routing::get,
};
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tower_http::services::ServeDir;

/// Live reload message type.
#[derive(Debug, Clone)]
pub enum ReloadMessage {
    /// Full page reload.
    Reload,
    /// CSS-only reload (hot reload).
    CssReload,
}

/// Server state containing the reload broadcaster.
#[derive(Clone)]
pub struct ServerState {
    /// Broadcast channel for live reload events.
    pub reload_tx: broadcast::Sender<ReloadMessage>,
}

impl ServerState {
    /// Create a new server state.
    pub fn new() -> Self {
        let (reload_tx, _) = broadcast::channel(16);
        Self { reload_tx }
    }

    /// Send a reload notification to all connected clients.
    pub fn notify_reload(&self) {
        let _ = self.reload_tx.send(ReloadMessage::Reload);
    }

    /// Send a CSS reload notification (for hot reload).
    #[allow(dead_code)]
    pub fn notify_css_reload(&self) {
        let _ = self.reload_tx.send(ReloadMessage::CssReload);
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the development server router.
pub fn create_router(output_dir: &Path, state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/__livereload", get(livereload_handler))
        .fallback_service(ServeDir::new(output_dir))
        .with_state(state)
}

/// Server-Sent Events handler for live reload.
async fn livereload_handler(
    axum::extract::State(state): axum::extract::State<Arc<ServerState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.reload_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        match msg {
            Ok(ReloadMessage::Reload) => Some(Ok(Event::default().data("reload"))),
            Ok(ReloadMessage::CssReload) => Some(Ok(Event::default().data("css-reload"))),
            Err(_) => None, // Ignore lagged messages
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}

/// JavaScript snippet to inject for live reload.
pub const LIVERELOAD_SCRIPT: &str = r#"
<script>
(function() {
    const source = new EventSource('/__livereload');
    source.onmessage = function(event) {
        if (event.data === 'reload') {
            window.location.reload();
        } else if (event.data === 'css-reload') {
            // Reload all CSS files
            document.querySelectorAll('link[rel="stylesheet"]').forEach(function(link) {
                const href = link.href.split('?')[0];
                link.href = href + '?v=' + Date.now();
            });
        }
    };
    source.onerror = function() {
        console.log('[livereload] Connection lost, retrying...');
    };
})();
</script>
"#;

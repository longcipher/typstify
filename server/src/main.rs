use std::path::PathBuf;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::{IntoResponse, Response as AxumResponse},
    routing::get,
};
use leptos::{logging::log, prelude::*};
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <link rel="stylesheet" href="/pkg/typstify.css" />
          <AutoReload options=options.clone() />
          <HydrationScripts options />
        </head>
        <body></body>
      </html>
    }
}

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).expect("failed to get configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let index_path = PathBuf::from(&*leptos_options.site_root).join("index.html");

    tokio::fs::write(index_path, shell(leptos_options.clone()).to_html())
        .await
        .expect("could not write index.html");

    let app = Router::new()
        .route("/", get(file_and_error_handler))
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind to address");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error while running");
}

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
) -> AxumResponse {
    let root = options.site_root.clone();
    match get_static_file(uri.clone(), &root).await {
        Ok(res) => res.into_response(),
        Err(_) => get_static_file(Uri::from_static("/index.html"), &root)
            .await
            .expect("could not find index.html")
            .into_response(),
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .expect("failed to build request");
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(Body::new)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}

use axum::{
    extract::{Host, Request},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use color_eyre::eyre::Context;
use setup::{setup_sentry, setup_tracing};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod server_tracing;
mod setup;

mod apis;

fn main() -> color_eyre::Result<()> {
    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?
        .block_on(async { _main().await })
}

async fn _main() -> color_eyre::Result<()> {
    setup_tracing()?;

    run_axum().await
}

#[derive(Clone, Debug)]
struct AppState;

async fn handler(Host(host): Host) -> Response {
    match host.as_str() {
        "redirects.coreyja.domains" => {
            "I have lots of domains. Some of them just redirect to others.".into_response()
        }
        _ => "This is not one of the hosts I know about.".into_response(),
    }
}

async fn host_redirection(
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    Host(host): Host,
    request: Request,
    next: axum::middleware::Next,
) -> Response {
    let redirect_to = match host.as_str() {
        "coreyja.tv" | "coreyja.tube" => Some("https://coreyja.com/videos"),
        "coreyja.blog" => Some("https://coreyja.com/posts"),
        "coreyja.club" => Some("https://discord.gg/CpAPpXrgUq"),
        _ => None,
    };

    if let Some(redirect_to) = redirect_to {
        return Redirect::to(redirect_to).into_response();
    };

    next.run(request).await
}

async fn porkbun_handler() -> impl IntoResponse {
    let config = apis::porkbun::Config::from_env().expect("Failed to get porkbun config");

    axum::Json(apis::porkbun::fetch_domains(config).await.unwrap())
}

async fn run_axum() -> color_eyre::Result<()> {
    let tracer = server_tracing::Tracer;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(tracer)
        .on_response(tracer);

    let state = AppState;

    let outer = axum::Router::new()
        .route("/", get(handler))
        .route("/porkbun", get(porkbun_handler))
        .with_state(state)
        .layer(trace_layer)
        .layer(axum::middleware::from_fn(host_redirection));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, outer)
        .await
        .wrap_err("Failed to run server")?;

    Ok(())
}

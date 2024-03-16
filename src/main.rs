use axum::{
    extract::{Host, Request},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use cja::app_state::AppState as _;
use miette::IntoDiagnostic;
use setup::{setup_sentry, setup_tracing};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

mod server_tracing;
mod setup;

mod apis;
mod cron;
mod jobs;

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

    let app_state = AppState::from_env().await?;

    cja::sqlx::migrate!().run(app_state.db()).await?;

    info!("Spawning Tasks");
    let futures = vec![
        tokio::spawn(run_axum(app_state.clone())),
        tokio::spawn(cja::jobs::worker::job_worker(app_state.clone(), jobs::Jobs)),
        tokio::spawn(cron::run_cron(app_state.clone())),
    ];
    info!("Tasks Spawned");

    futures::future::try_join_all(futures).await?;

    Ok(())
}

#[derive(Clone, Debug)]
struct AppState {
    db: sqlx::Pool<sqlx::Postgres>,
    cookie_key: cja::server::cookies::CookieKey,
}

impl cja::app_state::AppState for AppState {
    fn version(&self) -> &str {
        env!("VERGEN_GIT_SHA")
    }

    fn db(&self) -> &sqlx::PgPool {
        &self.db
    }

    fn cookie_key(&self) -> &cja::server::cookies::CookieKey {
        &self.cookie_key
    }
}

impl AppState {
    pub async fn from_env() -> color_eyre::Result<Self> {
        let pool = crate::setup::setup_db_pool().await?;

        let cookie_key = cja::server::cookies::CookieKey::from_env_or_generate()?;

        Ok(Self {
            db: pool,
            cookie_key,
        })
    }
}

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

async fn run_axum(app_state: AppState) -> miette::Result<()> {
    let tracer = server_tracing::Tracer;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(tracer)
        .on_response(tracer);

    let outer = axum::Router::new()
        .route("/", get(handler))
        .with_state(app_state)
        .layer(trace_layer)
        .layer(axum::middleware::from_fn(host_redirection));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, outer).await.into_diagnostic()?;

    Ok(())
}

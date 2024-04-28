use axum::{
    extract::{Host, Request},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use cja::{
    app_state::AppState as _,
    server::run_server,
    setup::{setup_sentry, setup_tracing},
};
use miette::{Context, IntoDiagnostic};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

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
    setup_tracing("domains")
        .wrap_err("Failed to setup tracing")
        .unwrap();

    let app_state = AppState::from_env().await?;

    cja::sqlx::migrate!().run(app_state.db()).await?;

    info!("Spawning Tasks");
    let mut futures = vec![
        tokio::spawn(run_server(routes(app_state.clone()))),
        tokio::spawn(cja::jobs::worker::job_worker(app_state.clone(), jobs::Jobs)),
    ];
    if std::env::var("CRON_DISABLED").unwrap_or_else(|_| "false".to_string()) != "true" {
        futures.push(tokio::spawn(cron::run_cron(app_state.clone())));
    }
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
        let pool = setup_db_pool().await.unwrap();

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
        "beta.coreyja.com" => Some("https://coreyja.com"),
        _ => None,
    };

    if let Some(redirect_to) = redirect_to {
        return Redirect::to(redirect_to).into_response();
    };

    next.run(request).await
}

fn routes(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/", get(handler))
        .with_state(app_state)
        .layer(axum::middleware::from_fn(host_redirection))
}

#[tracing::instrument(err)]
pub async fn setup_db_pool() -> miette::Result<PgPool> {
    const MIGRATION_LOCK_ID: i64 = 0xDB_DB_DB_DB_DB_DB_DB;

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .into_diagnostic()?;

    sqlx::query!("SELECT pg_advisory_lock($1)", MIGRATION_LOCK_ID)
        .execute(&pool)
        .await
        .into_diagnostic()?;

    sqlx::migrate!().run(&pool).await.into_diagnostic()?;

    let unlock_result = sqlx::query!("SELECT pg_advisory_unlock($1)", MIGRATION_LOCK_ID)
        .fetch_one(&pool)
        .await
        .into_diagnostic()?
        .pg_advisory_unlock;

    match unlock_result {
        Some(b) => {
            if b {
                tracing::info!("Migration lock unlocked");
            } else {
                tracing::info!("Failed to unlock migration lock");
            }
        }
        None => panic!("Failed to unlock migration lock"),
    }

    Ok(pool)
}

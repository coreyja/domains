use color_eyre::eyre::Context;
use setup::{setup_sentry, setup_tracing};
use tokio::net::TcpListener;

mod setup;

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

    run_axum(AppState).await
}

#[derive(Clone, Debug)]
struct AppState;

async fn run_axum(app_state: AppState) -> color_eyre::Result<()> {
    let app = axum::Router::new().with_state(app_state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app)
        .await
        .wrap_err("Failed to run server")?;

    Ok(())
}

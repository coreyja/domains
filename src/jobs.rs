use serde::{Deserialize, Serialize};

use crate::AppState;

cja::impl_job_registry!(AppState, NoopJob);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NoopJob;

#[async_trait::async_trait]
impl cja::jobs::Job<AppState> for NoopJob {
    const NAME: &'static str = "NoopJob";

    async fn run(&self, _app_state: AppState) -> miette::Result<()> {
        tracing::info!("Noop job ran");

        Ok(())
    }
}

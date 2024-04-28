use std::time::Duration;

use cja::cron::{CronRegistry, Worker};

use crate::{jobs::refresh_domains::RefreshDomains, AppState};

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register_job(RefreshDomains, Duration::from_secs(60 * 60));

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> miette::Result<()> {
    Ok(Worker::new(app_state, cron_registry()).run().await?)
}

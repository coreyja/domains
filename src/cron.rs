use std::time::Duration;

use cja::cron::{CronRegistry, Worker};

use crate::{
    jobs::{
        refresh_domain_nameservers::RefreshDomainsNameservers, refresh_domains::RefreshDomains,
    },
    AppState,
};

pub fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

pub fn one_day() -> Duration {
    Duration::from_secs(60 * 60 * 24)
}

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register_job(RefreshDomains, one_hour());
    registry.register_job(RefreshDomainsNameservers, one_day());

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> cja::Result<()> {
    Ok(Worker::new(app_state, cron_registry()).run().await?)
}

use crate::{jobs::refresh_domains::RefreshDomains, AppState};

pub mod refresh_domains;

cja::impl_job_registry!(AppState, RefreshDomains);

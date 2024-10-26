use refresh_domain_nameservers::{RefreshDomainNameservers, RefreshDomainsNameservers};

use crate::{jobs::refresh_domains::RefreshDomains, AppState};

pub mod refresh_domain_nameservers;
pub mod refresh_domains;

cja::impl_job_registry!(
    AppState,
    RefreshDomains,
    RefreshDomainsNameservers,
    RefreshDomainNameservers
);

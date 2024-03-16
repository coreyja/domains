-- Add migration script here
CREATE TABLE
  PorkbunDomains (
    porkbun_domain_id UUID PRIMARY KEY NOT NULL,
    auto_renew BOOLEAN NOT NULL,
    purchase_date TIMESTAMPTZ NOT NULL,
    domain TEXT NOT NULL,
    expire_date TIMESTAMPTZ NOT NULL,
    not_local BOOLEAN NOT NULL,
    security_lock BOOLEAN NOT NULL,
    status TEXT,
    tld TEXT NOT NULL,
    whois_privacy BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
  );

CREATE UNIQUE INDEX idx_porkbun_domains_on_domain ON PorkbunDomains (domain);

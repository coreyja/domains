-- Add migration script here
ALTER TABLE PorkbunDomains ADD COLUMN nameservers TEXT[] NOT NULL DEFAULT '{}';

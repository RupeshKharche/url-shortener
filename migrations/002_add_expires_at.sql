alter table urls add column expires_at text null;
create index idx_urls_expires_at on urls(expires_at);
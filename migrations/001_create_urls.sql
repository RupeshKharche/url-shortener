create table urls (
    id text primary key not null,
    code text not null unique,
    original_url text not null,
    created_at text not null default current_timestamp
);

create index idx_urls_code on urls(code);
CREATE TABLE users (
    email TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    phone TEXT,
    password TEXT NOT NULL,
    created_at integer(8) not null default (strftime('%s','now')),
    admin BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    synopsis TEXT NOT NULL,
    location TEXT,
    created_at integer(8) not null default (strftime('%s','now')),
    closed_at integer(8),
    created_by TEXT REFERENCES users(id),
    closed_by TEXT REFERENCES users(id)
);

-- TODO: Caller information

CREATE TABLE comments (
    id TEXT PRIMARY KEY,
    job_id TEXT REFERENCES jobs(id),
    comment TEXT NOT NULL,
    created_at integer(8) not null default (strftime('%s','now')),
    created_by TEXT REFERENCES users(id)
);

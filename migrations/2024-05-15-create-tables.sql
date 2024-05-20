CREATE TABLE users (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    email TEXT UNIQUE,
    phone TEXT,
    password TEXT,
    created_at integer(8) not null default (strftime('%s','now')),
    admin BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    dispatcher BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    synopsis TEXT NOT NULL,
    location TEXT,
    caller_name TEXT,
    caller_phone TEXT,
    created_at integer(8) not null default (strftime('%s','now')),
    closed_at integer(8),
    created_by TEXT REFERENCES users(id),
    closed_by TEXT REFERENCES users(id)
);

CREATE TABLE comments (
    id TEXT PRIMARY KEY,
    job_id TEXT REFERENCES jobs(id),
    comment TEXT NOT NULL,
    created_at integer(8) not null default (strftime('%s','now')),
    created_by TEXT REFERENCES users(id)
);

CREATE TABLE resources (
    id TEXT PRIMARY KEY,
    display_name TEXT UNIQUE NOT NULL,
    in_service BOOLEAN NOT NULL DEFAULT false,
    comment TEXT
);

CREATE TABLE resource_user_bindings (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL REFERENCES resources(id),
    user_id TEXT NOT NULL REFERENCES resources(id),
    created_at integer(8) not null default (strftime('%s','now')),
    removed_at integer(8)
);

CREATE TABLE assignments (
    id TEXT PRIMARY KEY,
    resource_id TEXT REFERENCES resources(id),
    job_id TEXT REFERENCES jobs(id),
    assigned_at integer(8) not null default (strftime('%s','now')),
    removed_at integer(8),
    assigned_by TEXT REFERENCES users(id),
    removed_by TEXT REFERENCES users(id)
);

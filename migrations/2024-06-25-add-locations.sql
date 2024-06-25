CREATE TABLE resource_locations (
    resource_id TEXT REFERENCES resources(id),
    at_time integer(8) not null default (strftime('%s','now')),
    latitude TEXT NOT NULL,
    longitude TEXT NOT NULL,
    PRIMARY KEY (resource_id, at_time)
);

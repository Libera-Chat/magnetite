DROP TABLE IF EXISTS metadata;
DROP TABLE IF EXISTS hashes;

CREATE TABLE metadata (
    key VARCHAR(16) UNIQUE,
    value VARCHAR(256)
);

CREATE TABLE hashes (
    index INTEGER,
    hash VARCHAR(256)
);
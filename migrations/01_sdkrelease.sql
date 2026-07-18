CREATE TABLE IF NOT EXISTS sdk_releases
(
    id           UUID PRIMARY KEY DEFAULT uuidv7(),
    release_date TIMESTAMP WITH TIME ZONE NOT NULL,
    changelog    TEXT
);

CREATE INDEX IF NOT EXISTS sdk_releases_date_idx
    ON sdk_releases (release_date DESC);

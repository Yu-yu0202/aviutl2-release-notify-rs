CREATE TABLE IF NOT EXISTS aviutl2_releases (
    id UUID PRIMARY KEY default uuidv7(),
    version VARCHAR(50) NOT NULL,
    zip_url TEXT NOT NULL,
    exe_url TEXT NOT NULL,
    release_date TIMESTAMP WITH TIME ZONE NOT NULL,
    aviutl2_changelog TEXT,
    lua_changelog TEXT
);

CREATE INDEX IF NOT EXISTS aviutl2_releases_date_idx
    ON aviutl2_releases (release_date DESC);

ALTER TABLE aviutl2_releases
    ADD CONSTRAINT aviutl2_releases_version_unique UNIQUE (version);

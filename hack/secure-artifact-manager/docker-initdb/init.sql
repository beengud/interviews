CREATE TABLE IF NOT EXISTS artifacts (
    id TEXT PRIMARY KEY,
    original_filename TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256 TEXT NOT NULL,
    upload_time TEXT NOT NULL,
    metadata TEXT NOT NULL,
    scan_status TEXT NOT NULL DEFAULT 'in-progress'
);
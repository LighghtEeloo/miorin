-- Raw entries table
CREATE TABLE IF NOT EXISTS raw_entries (
    id TEXT PRIMARY KEY,
    tags_json TEXT NOT NULL, -- JSON serialized Vec<String>
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    vibe_json TEXT, -- JSON serialized Vibe, nullable
    source_json TEXT NOT NULL, -- JSON serialized RawSource
    content_json TEXT NOT NULL -- JSON serialized RawContent
);

-- Cubes table
CREATE TABLE IF NOT EXISTS cubes (
    id TEXT PRIMARY KEY,
    tags_json TEXT NOT NULL, -- JSON serialized Vec<String>
    pin INTEGER NOT NULL DEFAULT 0, -- 0 or 1 for boolean
    content_json TEXT NOT NULL, -- JSON serialized CubeContent
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    vibe_json TEXT -- JSON serialized Vibe, nullable
);

-- Links table (for links between cubes)
CREATE TABLE IF NOT EXISTS links (
    id TEXT PRIMARY KEY,
    tags_json TEXT NOT NULL, -- JSON serialized Vec<String>
    from_cube_id TEXT NOT NULL,
    to_cube_id TEXT NOT NULL,
    kind_json TEXT NOT NULL, -- JSON serialized LinkKind
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    vibe_json TEXT, -- JSON serialized Vibe, nullable
    FOREIGN KEY (from_cube_id) REFERENCES cubes(id) ON DELETE CASCADE,
    FOREIGN KEY (to_cube_id) REFERENCES cubes(id) ON DELETE CASCADE
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_raw_entries_created_at ON raw_entries(created_at);
CREATE INDEX IF NOT EXISTS idx_cubes_pin ON cubes(pin);
CREATE INDEX IF NOT EXISTS idx_cubes_created_at ON cubes(created_at);
CREATE INDEX IF NOT EXISTS idx_links_from_cube ON links(from_cube_id);
CREATE INDEX IF NOT EXISTS idx_links_to_cube ON links(to_cube_id);


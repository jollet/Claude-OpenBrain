-- 001_initial.sql

CREATE TABLE thoughts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    has_embedding BOOLEAN DEFAULT 0
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE thought_tags (
    thought_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (thought_id, tag_id),
    FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- sqlite-vec setup for embeddings (enabled in later phases, but good to prep)
CREATE VIRTUAL TABLE vec_thoughts USING vec0(
    thought_id INTEGER PRIMARY KEY,
    embedding float[384] -- Dimension for all-MiniLM-L6-v2 / nomic-embed-text
);

CREATE TABLE IF NOT EXISTS posts (
    id TEXT PRIMARY KEY,
    storage_id TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    content TEXT NOT NULL,
    excerpt TEXT,
    author TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    category TEXT,
    UNIQUE(content_hash)
);

CREATE INDEX idx_posts_slug ON posts(slug);
CREATE INDEX idx_posts_published ON posts(published);
CREATE INDEX idx_posts_created_at ON posts(created_at);
CREATE INDEX idx_posts_author ON posts(author);
CREATE INDEX idx_posts_category ON posts(category);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS post_tags (
    post_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS posts_fts USING fts5(
    title,
    content,
    excerpt,
    author,
    content=posts,
    content_rowid=rowid
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER posts_ai AFTER INSERT ON posts BEGIN
    INSERT INTO posts_fts(rowid, title, content, excerpt, author)
    VALUES (new.rowid, new.title, new.content, new.excerpt, new.author);
END;

CREATE TRIGGER posts_ad AFTER DELETE ON posts BEGIN
    DELETE FROM posts_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER posts_au AFTER UPDATE ON posts BEGIN
    DELETE FROM posts_fts WHERE rowid = old.rowid;
    INSERT INTO posts_fts(rowid, title, content, excerpt, author)
    VALUES (new.rowid, new.title, new.content, new.excerpt, new.author);
END;
CREATE TABLE attachments (
    id SERIAL PRIMARY KEY,
    file_path VARCHAR NOT NULL,
    thumbnail_path VARCHAR NOT NULL,
    item_id INTEGER REFERENCES items(id),
    uploader_id INTEGER NOT NULL REFERENCES users(id),
    uploaded_at TIMESTAMP WITH TIME ZONE NOT NULL
);

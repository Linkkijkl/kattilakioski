CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES items(id),
    buyer_id INTEGER NOT NULL REFERENCES users(id),
    item_amount INTEGER NOT NULL,
    transacted_at TIMESTAMP WITH TIME ZONE NOT NULL
)

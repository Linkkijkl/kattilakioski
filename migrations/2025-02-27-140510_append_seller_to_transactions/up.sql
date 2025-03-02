ALTER TABLE transactions RENAME COLUMN buyer_id TO payer_id;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS receiver_id INTEGER REFERENCES users(id);
ALTER TABLE transactions ALTER COLUMN item_id DROP NOT NULL;
ALTER TABLE transactions ALTER COLUMN payer_id DROP NOT NULL;
ALTER TABLE transactions ALTER COLUMN item_amount SET DEFAULT 0;

/*
Transfer newly created required column `receiver_id` from associated item (appointed by
`item_id`, from item table's column `seller_id`)
*/
UPDATE transactions t
SET receiver_id = i.seller_id
FROM items i
WHERE t.item_id = i.id;

/*
Make receiver id non-nullable *after* transferring data, to keep relation intact
*/
ALTER TABLE transactions ALTER COLUMN receiver_id SET NOT NULL;

ALTER TABLE transactions ALTER COLUMN payer_id SET NOT NULL;
ALTER TABLE transactions RENAME COLUMN payer_id TO buyer_id;
ALTER TABLE transactions DROP COLUMN receiver_id;
ALTER TABLE transactions ALTER COLUMN item_id SET NOT NULL;
ALTER TABLE transactions ALTER COLUMN item_amount DROP DEFAULT;

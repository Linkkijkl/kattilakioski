ALTER TABLE transactions ADD COLUMN IF NOT EXISTS amount_cents INTEGER NOT NULL;

/*
Transfer newly created required column `amount_cents` from associated item (appointed by `item_id`,
from item table's column `price_cents`).
*/
UPDATE transactions t
SET amount_cents = i.price_cents
FROM items i
WHERE t.item_id = i.id;

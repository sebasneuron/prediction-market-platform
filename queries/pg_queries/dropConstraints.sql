SELECT conname
FROM pg_constraint
WHERE conrelid = 'polymarket.orders'::regclass
  AND contype = 'c'
  AND pg_get_constraintdef(oid) ILIKE '%price%';

ALTER TABLE polymarket.orders DROP CONSTRAINT orders_price_check;
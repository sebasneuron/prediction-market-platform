-- truncate table polymarket.markets CASCADE;

-- select * from polymarket.markets where status = 'open'::polymarket.market_status;

-- SELECT 
--             o.id, o.user_id, o.market_id,
--             o.outcome as "outcome: Outcome",
--             o.price, o.quantity, o.filled_quantity,
--             o.status as "status: OrderStatus",
--             o.side as "side: OrderSide",
--             o.created_at, o.updated_at, m.liquidity_b
--             FROM polymarket.orders o
--             LEFT JOIN polymarket.markets m ON o.market_id = m.id
--             WHERE o.status = 'open'::polymarket.order_status 


-- UPDATE polymarket.markets
-- 	SET status = 'closed'::polymarket.market_status, updated_at = CURRENT_TIMESTAMP
--     WHERE id = 'c3cc74a7-6695-41e1-8bdf-d5affa5b4aac'::uuid AND market_expiry <= CURRENT_TIMESTAMP AND status = 'open'::polymarket.market_status;

select * from polymarket.markets ORDER BY created_at DESC;

-- delete from polymarket.markets where id = '06761131-a639-4dae-9dff-d652ff3b3832'::uuid;

-- SELECT
--   market_id,
--   SUM(quantity * price) AS notional_volume
-- FROM
--   polymarket.user_trades
-- GROUP BY
--   market_id;
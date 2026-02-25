-- truncate table polymarket.user_trades;

select * from polymarket.user_trades order by created_at DESC;

-- SELECT
--     market_id,
--     SUM(quantity) AS total_volume
-- FROM polymarket.user_trades
-- WHERE timestamp >= NOW() - INTERVAL '6 hours'
-- GROUP BY market_id;

-- select t.id ,u.name, u.email, u.avatar, t.trade_type, t.outcome, t.price, t.quantity, t.timestamp
-- FROM polymarket.user_trades t
-- RIGHT JOIN polymarket.users u ON u.id = t.user_id
-- WHERE u.name != 'Admin' AND t.market_id = '20ec3758-04ef-4300-a24c-c9019cf55c95'::uuid
-- ORDER BY t.timestamp DESC;

-- select 
-- 	m.name as market_name,
-- 	m.logo as market_logo,
-- 	m.status as market_status,
-- 	m.final_outcome as market_final_outcome,

-- 	t.trade_type,
-- 	t.outcome as trade_outcome,
-- 	t.price as trade_price,
-- 	t.quantity as trade_quantity
-- FROM polymarket.user_trades t
-- JOIN polymarket.markets m ON t.market_id = m.id
-- ORDER BY t.created_at DESC;

	

-- paginated

SELECT 
	m.name AS market_name,
	m.logo AS market_logo,
	m.status AS "market_status: MarketStatus",
	m.final_outcome AS "market_final_outcome: Outcome",

	t.trade_type AS "trade_type: OrderSide",
	t.outcome AS "trade_outcome: Outcome",
	t.price AS trade_price,
	t.quantity AS trade_quantity
FROM polymarket.user_trades t
JOIN polymarket.markets m ON t.market_id = m.id
WHERE t.user_id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid
ORDER BY t.timestamp DESC
LIMIT 10 OFFSET 1;
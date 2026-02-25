WITH 
holdings AS (
    SELECT 
        uh.market_id,
        uh.outcome,
        uh.shares
    FROM polymarket.user_holdings uh
    WHERE uh.user_id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid
),

orders AS (
    SELECT 
        COUNT(*) FILTER (WHERE status = 'open') AS open_orders,
        COUNT(*) FILTER (WHERE status = 'partial_fill') AS partial_orders,
        COUNT(*) AS total_orders,
        AVG(filled_quantity / NULLIF(quantity, 0)) AS avg_fill_ratio
    FROM polymarket.orders
    WHERE user_id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid
),

trades AS (
    SELECT 
        COUNT(*) AS total_trades,
        SUM(quantity) AS total_volume,
        AVG(price) AS avg_trade_price,
        MAX(quantity) AS max_trade_qty,
        MIN(created_at) AS first_trade_at,
        MAX(created_at) AS last_trade_at,
        COUNT(DISTINCT market_id) AS markets_traded
    FROM polymarket.user_trades
    WHERE user_id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid
),

txns AS (
    SELECT
        SUM(amount) FILTER (WHERE transaction_type = 'deposit') AS total_deposit,
        SUM(amount) FILTER (WHERE transaction_type = 'withdrawal') AS total_withdraw,
        MAX(created_at) FILTER (WHERE transaction_type = 'deposit') AS last_deposit,
        MAX(created_at) FILTER (WHERE transaction_type = 'withdrawal') AS last_withdraw
    FROM polymarket.user_transactions
    WHERE user_id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid
)

SELECT
    u.id,
    u.name,
    u.email,
    u.avatar,
    u.public_key,
    u.balance,
    u.last_login,
    u.created_at,
    
    -- Orders
    o.open_orders,
    o.partial_orders,
    o.total_orders,
    o.avg_fill_ratio,

    -- Trades
    t.total_trades,
    t.total_volume,
    t.avg_trade_price,
    t.max_trade_qty,
    t.first_trade_at,
    t.last_trade_at,
    t.markets_traded,

    -- Txns
    x.total_deposit,
    x.total_withdraw,
    x.last_deposit,
    x.last_withdraw

FROM polymarket.users u
LEFT JOIN orders o ON true
LEFT JOIN trades t ON true
LEFT JOIN txns x ON true
WHERE u.id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid;
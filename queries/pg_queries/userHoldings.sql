-- truncate table polymarket.user_holdings;

-- delete from  polymarket.user_holdings where id != '8c848e79-c2d2-46e5-bdac-f6af89e59afb'::uuid;


-- INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
--             VALUES ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, 'bd609b17-d3d3-4f70-a5e2-0a3f3aa2160c'::uuid, -10)
--             ON CONFLICT (user_id, market_id)
-- 			DO UPDATE SET shares = polymarket.user_holdings.shares + -10,
--             updated_at = NOW()
--             RETURNING id, user_id, market_id, shares, created_at, updated_at;
			

-- INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
--             VALUES ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, 'bd609b17-d3d3-4f70-a5e2-0a3f3aa2160c'::uuid, 200)
--             ON CONFLICT (user_id, market_id) DO NOTHING;

-- SELECT
--     uh.market_id,
--     uh.outcome,
--     uh.shares,
    
--     m.name AS market_name,
--     m.description AS market_description,
--     m.logo AS market_logo,
--     m.status AS market_status,
--     m.final_outcome,
--     m.market_expiry,
--     m.created_at AS market_created_at,
--     m.updated_at AS market_updated_at

-- FROM polymarket.user_holdings uh
-- JOIN polymarket.markets m ON uh.market_id = m.id
-- WHERE uh.user_id = 'cf2f0f54-f66e-4a61-bc85-9a26653e77e9'::uuid;



select * from polymarket.user_holdings order by created_at DESC;

select 
	uh.user_id,
	uh.shares,
	uh.outcome,
	u.balance,
	u.email
FROM 
	polymarket.user_holdings uh
JOIN
	polymarket.users u ON uh.user_id = u.id
ORDER BY
	uh.created_at DESC;
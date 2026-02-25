select * from cron.job;

SELECT * FROM cron.job_run_details ORDER BY start_time DESC LIMIT 10;


̇
-- CREATE OR REPLACE FUNCTION polymarket.close_market(market_id UUID)
-- RETURNS VOID AS $$
-- BEGIN
--     UPDATE polymarket.markets
--     SET status = 'closed'::polymarket.market_status, updated_at = CURRENT_TIMESTAMP
--     WHERE id = market_id AND market_expiry <= CURRENT_TIMESTAMP AND status = 'open'::polymarket.market_status;
-- END;
-- $$ LANGUAGE plpgsql;

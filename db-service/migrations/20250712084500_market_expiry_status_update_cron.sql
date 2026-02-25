-- Add migration script here

CREATE OR REPLACE FUNCTION polymarket.close_market(market_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE polymarket.markets
    SET status = 'closed'::polymarket.market_status, updated_at = CURRENT_TIMESTAMP
    WHERE id = market_id
        AND status = 'open'::polymarket.market_status;

    -- Remove the cron job for this market
    PERFORM cron.unschedule('close_market_job_' || market_id::text);
END;
$$ LANGUAGE plpgsql;

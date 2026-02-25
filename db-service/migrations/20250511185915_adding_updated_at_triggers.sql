-- Add migration script here


-- function to update the updated_at field
CREATE OR REPLACE FUNCTION polymarket.set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- triggers

-- users
CREATE TRIGGER set_updated_at_users_trigger 
BEFORE UPDATE ON "polymarket"."users"
FOR EACH ROW
EXECUTE FUNCTION polymarket.set_updated_at();

-- markets
CREATE TRIGGER set_updated_at_markets_trigger 
BEFORE UPDATE ON "polymarket"."markets"
FOR EACH ROW
EXECUTE FUNCTION polymarket.set_updated_at();

-- orders
CREATE TRIGGER set_updated_at_orders_trigger
BEFORE UPDATE ON "polymarket"."orders"
FOR EACH ROW
EXECUTE FUNCTION polymarket.set_updated_at();

-- user_trades
CREATE TRIGGER set_updated_at_user_trades_trigger
BEFORE UPDATE ON "polymarket"."user_trades"
FOR EACH ROW
EXECUTE FUNCTION polymarket.set_updated_at();

-- user_holdings
CREATE TRIGGER set_updated_at_user_holdings_trigger
BEFORE UPDATE ON "polymarket"."user_holdings"
FOR EACH ROW
EXECUTE FUNCTION polymarket.set_updated_at();

-- user_transactions
CREATE TRIGGER set_updated_at_user_transactions_trigger
BEFORE UPDATE ON "polymarket"."user_transactions"
FOR EACH ROW
EXECUTE FUNCTION polymarket.set_updated_at();




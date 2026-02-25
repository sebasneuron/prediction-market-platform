-- Add migration script here

-- enums
CREATE TYPE polymarket.market_status AS ENUM ('open', 'closed', 'settled');
CREATE TYPE polymarket.outcome AS ENUM ('yes', 'no', 'unspecified');
CREATE TYPE polymarket.order_side AS ENUM ('buy', 'sell');
CREATE TYPE polymarket.order_status AS ENUM ('open', 'filled', 'cancelled', 'unspecified', 'expired', 'pending_cancel', 'partial_fill', 'pending_update');
CREATE TYPE polymarket.user_transaction_type AS ENUM ('deposit', 'withdrawal', 'trade');
CREATE TYPE polymarket.user_transaction_status AS ENUM ('pending', 'complete', 'failed');
CREATE TYPE polymarket.order_type AS ENUM ('limit','market', 'stop_loss', 'take_profit');


-- users
CREATE TABLE IF NOT EXISTS polymarket.users (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    -- oAuth2 fields
    "google_id" varchar(255) UNIQUE NOT NULL,
    "email" varchar(255) UNIQUE NOT NULL,
    "name" varchar(255) NOT NULL,
    "avatar" varchar(255) NOT NULL,
    "last_login" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- wallet fields
    "public_key" varchar(255) NOT NULL UNIQUE,
    "private_key" TEXT NOT NULL UNIQUE,
    "balance" decimal(20,8) NOT NULL DEFAULT 0,            
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- markets
CREATE TABLE IF NOT EXISTS polymarket.markets (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "name" varchar(255) NOT NULL,
    "description" text NOT NULL,
    "logo" varchar(255) NOT NULL,
    "status" polymarket.market_status NOT NULL DEFAULT 'open',
    "liquidity_b" decimal NOT NULL DEFAULT 0,
    "final_outcome" polymarket.outcome NOT NULL DEFAULT 'unspecified',
    "market_expiry" timestamp NOT NULL,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- orders
CREATE TABLE IF NOT EXISTS polymarket.orders (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),
    "side" polymarket.order_side NOT NULL,
    "outcome" polymarket.outcome NOT NULL DEFAULT 'unspecified',
    "price" decimal NOT NULL, -- for market orders, the price is the user's budget
    "quantity" decimal NOT NULL CHECK ("quantity" >= 0),
    "filled_quantity" decimal NOT NULL DEFAULT 0,
    "status" polymarket.order_status NOT NULL DEFAULT 'unspecified',
    "order_type" polymarket.order_type NOT NULL,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- user_trades
CREATE TABLE IF NOT EXISTS polymarket.user_trades (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "buy_order_id" uuid NOT NULL REFERENCES polymarket.orders("id"),
    "sell_order_id" uuid NOT NULL REFERENCES polymarket.orders("id"),
    "trade_type" polymarket.order_side NOT NULL, -- we are storing this to prevent joins for optimizing query performance (for order.type == buy then trade_type == sell and vice versa)
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),
    "outcome" polymarket.outcome NOT NULL,
    "price" decimal NOT NULL,
    "quantity" decimal NOT NULL CHECK ("quantity" > 0),
    "timestamp" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- user_holdings
CREATE TABLE IF NOT EXISTS polymarket.user_holdings (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "market_id" uuid NOT NULL REFERENCES polymarket.markets("id"),    
    "shares" decimal NOT NULL DEFAULT 0,
    "outcome" polymarket.outcome NOT NULL DEFAULT 'unspecified',
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (user_id, market_id, outcome)
);

-- user_transactions
CREATE TABLE IF NOT EXISTS polymarket.user_transactions (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES polymarket.users("id"),
    "amount" decimal NOT NULL CHECK ("amount" > 0),
    "transaction_type" polymarket.user_transaction_type NOT NULL,
    "transaction_status" polymarket.user_transaction_status NOT NULL,
    "tx_hash" varchar(255) NOT NULL,
    "confirmed_at" timestamp,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);
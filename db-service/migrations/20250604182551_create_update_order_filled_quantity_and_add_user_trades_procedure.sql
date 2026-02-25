-- Add migration script here

-- This procedure is used to post update the opposite orders which are filled by the current order.

CREATE OR REPLACE PROCEDURE polymarket.update_order_and_process_trade(
    in_current_order_id UUID, -- order which is to match
    in_opposite_order_id UUID, -- order which was matched with above order
    in_new_filled_quantity DECIMAL
) LANGUAGE plpgsql
AS $$ 
DECLARE 
    -- declare necessary variables
    v_user_id UUID;
    v_market_id UUID;
    v_outcome polymarket.outcome;
    v_side polymarket.order_side;
    v_price DECIMAL;
    v_quantity DECIMAL;
    v_total_filled DECIMAL;
    v_status polymarket.order_status;
    v_new_total_filled DECIMAL;
BEGIN
    -- Get order details
    SELECT user_id, market_id, outcome, side, price, quantity, filled_quantity
    INTO v_user_id, v_market_id, v_outcome, v_side, v_price, v_quantity, v_total_filled
    FROM polymarket.orders
    WHERE id = in_opposite_order_id
    FOR UPDATE;

    -- update filled quantity
    v_new_total_filled := v_total_filled + in_new_filled_quantity;

    -- validate total balance
    IF v_new_total_filled > v_quantity THEN
        RAISE EXCEPTION 'Filled quantity (%.2f) exceeds total quantity (%.2f)', v_new_total_filled, v_quantity;
    END IF;

    -- update status
    IF v_total_filled >= v_quantity THEN
        v_status := 'filled'::polymarket.order_status;    
    END IF;

    -- update order
    UPDATE polymarket.orders
    SET filled_quantity = v_new_total_filled,
        status = v_status
    WHERE id = in_opposite_order_id;

    IF v_new_total_filled = v_quantity THEN
        -- insert new trade 
        INSERT INTO polymarket.user_trades (
            user_id,
            buy_order_id,
            sell_order_id,
            market_id,
            outcome,
            price,
            quantity
        ) VALUES (
            v_user_id,
            in_current_order_id,
            in_opposite_order_id,
            v_market_id,
            v_outcome,
            v_price,
            in_new_filled_quantity
        );

        -- update or insert into user holdings
        INSERT INTO polymarket.user_holdings (
            user_id,
            market_id,
            outcome,
            shares
        ) 
        VALUES (
            v_user_id,
            v_market_id,
            v_outcome,
            v_new_total_filled
        ) ON CONFLICT (user_id, market_id, outcome)
        DO UPDATE SET
         shares = polymarket.user_holdings.shares + in_new_filled_quantity;
    END IF;
END;
$$;
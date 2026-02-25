/*
 * This calculation is based on 0.0 decimal precision.
 * Means 0.3 is 0.3 here, not 30. That thing must be handled by above level data processing
 *
 * Price for every incoming order must be between 0 to 1 (inclusive).
 *
 *
 * ## Market order rules
 *
 * 1. Partial matches are not allowed.
 * 2. If order is partially filled, then it's going to be cancelled.
 * 3. Market orders are not going to be added in the order book.
 */

use std::collections::BTreeMap;

use db_service::schema::{
    enums::{OrderSide, OrderStatus, OrderType},
    orders::Order,
};
use rust_decimal::Decimal;
use utility_helpers::{
    log_error, log_info,
    types::{OrderBookDataStruct, OrderLevel},
};
use uuid::Uuid;

#[derive(Default, Debug)]
pub(crate) struct PriceLevel {
    pub(crate) orders: Vec<OrderBookEntry>, // should I consider using hashmap here for O(1) lookup
    pub(crate) total_quantity: Decimal,
}

#[derive(Debug)]
pub(crate) struct OrderBookEntry {
    pub user_id: Uuid,
    pub order_id: Uuid,
    pub total_quantity: Decimal,
    pub filled_quantity: Decimal,
}

#[derive(Debug, Default)]
pub(crate) struct OutcomeBook {
    pub(crate) bids: BTreeMap<Decimal, PriceLevel>, // buyers side
    pub(crate) asks: BTreeMap<Decimal, PriceLevel>, // sellers side
}

#[derive(Debug)]
pub(crate) struct OrderBookMatchedOutput {
    pub order_id: Uuid,
    pub opposite_order_id: Uuid,
    pub matched_quantity: Decimal,
    pub price: Decimal,
    pub opposite_order_total_quantity: Decimal,
    pub opposite_order_filled_quantity: Decimal,
}

impl OutcomeBook {
    pub(crate) fn add_order(&mut self, order: &Order) {
        if order.status == OrderStatus::FILLED || order.status == OrderStatus::CANCELLED {
            log_info!(
                "Order with id {} is already filled or cancelled, not adding to book",
                order.id
            );
            return; // no need to add filled or cancelled orders
        }

        if order.price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}, not adding order",
                order.price
            );
            return; // price should be less than or equal to 1.0 (or 100%)
        }
        let side = match order.side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };

        let price_level = side.entry(order.price).or_default();

        let entry = OrderBookEntry {
            filled_quantity: order.filled_quantity,
            order_id: order.id,
            total_quantity: order.quantity,
            user_id: order.user_id,
        };

        price_level.orders.push(entry);
        price_level.total_quantity += order.quantity - order.filled_quantity;
    }

    pub(super) fn best_bid(&self) -> Option<Decimal> {
        // sorted in ascending order, so we take the last one (highest available price from buyers to sellers)
        self.bids.keys().next_back().cloned()
    }

    pub(super) fn best_ask(&self) -> Option<Decimal> {
        // keys are sorted in ascending order, so lowest price from sellers to buyers is first
        self.asks.keys().next().cloned()
    }

    pub(super) fn remove_order(&mut self, order_id: Uuid, side: OrderSide, price: Decimal) -> bool {
        if price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}",
                price
            );
            return false; // price should be less than or equal to 1.0 (or 100%)
        }
        let price_side = match side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };
        if let Some(price_level) = price_side.get_mut(&price) {
            if let Some(pos) = price_level
                .orders
                .iter()
                .position(|order| order.order_id == order_id)
            {
                let removed_order = price_level.orders.remove(pos);
                price_level.total_quantity -=
                    removed_order.total_quantity - removed_order.filled_quantity;

                if price_level.orders.is_empty() {
                    price_side.remove(&price);
                }
                return true;
            }
        }
        false
    }

    // returns matched orders if updated order is matched with some order
    pub(super) fn update_order(
        &mut self,
        order: &mut Order,
        updated_price: Decimal,
        new_quantity: Decimal,
    ) -> bool {
        if order.price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}",
                order.price
            );
            return false; // invalid price
        }
        if order.quantity == new_quantity && order.price == updated_price {
            log_info!("No changes in order, nothing to update");
            return true; // no changes
        }
        // removing order
        if !self.remove_order(order.id, order.side, order.price) {
            log_info!("Order not found in book, cannot update");
            return false; // order not found
        }
        order.price = updated_price;
        order.quantity = new_quantity;
        order.status = OrderStatus::OPEN; // resetting status to open

        self.add_order(order);

        true
    }

    pub(super) fn _update_order_filled_quantity(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        current_price: Decimal,
        new_filled_quantity: Decimal,
    ) -> bool {
        let price_mapping = match side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };
        if let Some(price_level) = price_mapping.get_mut(&current_price) {
            if let Some(order) = price_level
                .orders
                .iter_mut()
                .find(|order| order.order_id == order_id)
            {
                /*
                   35 price_level.total_quantity

                   10 order.total_quantity (already exists)
                   5 order.filled_quantity
                   5 order.remaining_quantity

                   price_level.total_quantity = 30 (30 - 5)

                   update
                   10 -> order.total_quantity
                   5 -> order.filled_quantity
                   7 -> new_filled_quantity
                   prev_remaining = 10 - 5 = 5
                   new_remaining = 10 - 7 = 3

                   price_level.total_quantity = 30 + 3 - 5 = 28
                */
                let prev_remaining = order.total_quantity - order.filled_quantity;

                let new_remaining = order.total_quantity - new_filled_quantity;

                price_level.total_quantity =
                    price_level.total_quantity + new_remaining - prev_remaining;
                order.filled_quantity = new_filled_quantity;

                if price_level.total_quantity <= Decimal::ZERO {
                    price_mapping.remove(&current_price);
                }

                return true;
            }
        }

        false
    }

    /// This function modifies the current order and opposite orders. It's not used for simulation
    pub(super) fn match_order(&mut self, order: &mut Order) -> Vec<OrderBookMatchedOutput> {
        // order id, opposite order id, matched quantity, price
        let mut matches: Vec<OrderBookMatchedOutput> = Vec::new();

        if order.status != OrderStatus::OPEN {
            return matches; // only open orders can be matched
        }
        if order.price > Decimal::ONE && order.order_type != OrderType::MARKET {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}",
                order.price
            );
            return matches; // price should be less than or equal to 1.0 (or 100%)
        }
        if order.price != Decimal::ZERO && order.order_type != OrderType::MARKET {
            if order.quantity == Decimal::ZERO {
                order.status = OrderStatus::FILLED; // if quantity is zero, we consider it as filled
                order.filled_quantity = Decimal::ZERO; // no quantity to match

                log_info!("Order quantity is zero, nothing to match");
                return matches; // no quantity to match
            }
        }

        let (book, is_buy) = match order.side {
            OrderSide::BUY => (&mut self.asks, true), // inverse matching
            OrderSide::SELL => (&mut self.bids, false),
        };

        let mut keys: Vec<Decimal> = book.keys().cloned().collect(); // already sorted in ascending if is_buy true
        if is_buy {
            // still sorting in ascending (may be computer make mistake in case...) for buyers (best cheap price on top)
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            // keys.reverse(); // TODO: research on this from asc -> desc (reversing)
            keys.sort_by(|a, b| b.partial_cmp(a).unwrap()); // sorting in descending order (for sellers best expensive price on top)
        }

        let mut remaining = order.quantity - order.filled_quantity;
        if remaining <= Decimal::ZERO {
            return matches;
        }

        for price in keys {
            // case of market order
            if order.order_type != OrderType::MARKET {
                // bounds checking for limit orders
                if (is_buy && price > order.price) || (!is_buy && price < order.price) {
                    continue;
                }
            }

            if let Some(price_level) = book.get_mut(&price) {
                for opposite_order in price_level.orders.iter_mut() {
                    if order.id == opposite_order.order_id
                        || order.user_id == opposite_order.user_id
                    {
                        // skip matching with itself
                        continue;
                    }
                    let opp_remaining =
                        opposite_order.total_quantity - opposite_order.filled_quantity;
                    if opp_remaining <= Decimal::ZERO {
                        continue;
                    }

                    let match_qty = remaining.min(opp_remaining);

                    ///// ATOMIC Operation START (trusting on parking lot's RWLock )
                    opposite_order.filled_quantity += match_qty;

                    order.filled_quantity += match_qty;
                    remaining -= match_qty;

                    matches.push(OrderBookMatchedOutput {
                        order_id: order.id,
                        opposite_order_id: opposite_order.order_id,
                        matched_quantity: match_qty,
                        price,
                        // price: opposite_order.price, // price of matching order
                        opposite_order_total_quantity: opposite_order.total_quantity,
                        opposite_order_filled_quantity: opposite_order.filled_quantity,
                    });

                    if remaining == Decimal::ZERO {
                        break;
                    }
                    ///// ATOMIC Operation END
                }

                // removing orders (keep only those which are not fully filled)
                price_level
                    .orders
                    .retain(|o| o.filled_quantity < o.total_quantity);

                price_level.total_quantity = price_level
                    .orders
                    .iter()
                    .map(|o| o.total_quantity - o.filled_quantity)
                    .sum();

                if price_level.orders.is_empty() {
                    book.remove(&price);
                }
                if remaining == Decimal::ZERO {
                    break;
                }
            }
        }

        if order.filled_quantity == order.quantity {
            order.status = OrderStatus::FILLED;
        }

        matches
    }

    pub(crate) fn create_market_order(
        &mut self,
        order: &mut Order,
        budget: Decimal,
    ) -> Vec<OrderBookMatchedOutput> {
        if order.order_type != OrderType::MARKET {
            log_error!("Order type must be MARKET for create_market_order function");
            return Vec::new(); // only market orders can be created here
        }
        // This function is used to create a market order, which will match with the best available orders in the book
        // It will not check the price of the order, but will match with the best available orders until the quantity is filled or no more orders are available
        // NOTE: Market orders are not added in the order book
        let order_quantity = self.get_available_match_quantity(order, budget);

        order.quantity = order_quantity; // update order quantity to the available match quantity
        order.price = budget; // market orders do not have a price
        self.match_order(order)
    }

    // Getters ///

    pub(crate) fn get_order_book(&self) -> OrderBookDataStruct {
        let bids = &self.bids;
        let asks = &self.asks;

        let mut bids_values = Vec::new();
        let mut asks_values = Vec::new();

        for (price, level) in bids {
            let data = OrderLevel {
                price: *price,
                shares: level.total_quantity,
                users: level.orders.len(),
            };
            if level.orders.is_empty() {
                continue; // skip empty levels
            }
            bids_values.push(data);
        }
        for (price, level) in asks {
            let data = OrderLevel {
                price: *price,
                shares: level.total_quantity,
                users: level.orders.len(),
            };
            if level.orders.is_empty() {
                continue; // skip empty levels
            }
            asks_values.push(data);
        }

        OrderBookDataStruct {
            bids: bids_values,
            asks: asks_values,
        }
    }

    fn get_available_match_quantity(&mut self, order: &mut Order, budget: Decimal) -> Decimal {
        let mut available_quantity = Decimal::ZERO;
        if order.price != Decimal::ZERO {
            log_error!(
                "Market order price should be zero, but got: {}",
                order.price
            );
            return available_quantity; // market order price should be zero
        }

        let book = match order.side {
            OrderSide::BUY => &mut self.asks,  // match against asks
            OrderSide::SELL => &mut self.bids, // match against bids
        };

        let mut keys: Vec<Decimal> = book.keys().cloned().collect();

        if order.side == OrderSide::BUY {
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap()); // ascending: buy from lowest
        } else {
            keys.sort_by(|a, b| b.partial_cmp(a).unwrap()); // descending: sell to highest
        }

        let mut remaining_budget = budget;

        for price in keys {
            if remaining_budget <= Decimal::ZERO {
                // order is fully matched
                break;
            }

            if let Some(level) = book.get(&price) {
                let mut total_level_qty = Decimal::ZERO;

                for entry in &level.orders {
                    if entry.user_id == order.user_id {
                        // skip matching with itself
                        continue;
                    }
                    let rem_qty = entry.total_quantity - entry.filled_quantity;
                    if rem_qty > Decimal::ZERO {
                        total_level_qty += rem_qty;
                    }
                }

                let cost_to_consume_level = price * total_level_qty;

                if remaining_budget >= cost_to_consume_level {
                    // consume full level
                    available_quantity += total_level_qty;
                    remaining_budget -= cost_to_consume_level;
                } else {
                    // partial consume
                    let partial_qty = remaining_budget / price;
                    available_quantity += partial_qty;
                    remaining_budget -= price * partial_qty;
                    break;
                }
            }
        }

        if remaining_budget > Decimal::ZERO || budget == Decimal::ZERO {
            // order is not fully matched, but we have remaining budget
            order.status = OrderStatus::CANCELLED;
        }

        available_quantity
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;
    use db_service::schema::enums::{OrderType, Outcome};
    use rust_decimal_macros::dec;

    use super::*;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_market_order_custom() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),    // 0.25 -> 25$
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        }; // 0.25 buy - 10 qty (buy)
        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),   // 0.20 -> 20$
            quantity: Decimal::new(3, 0), // 3
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id,
            order_type: OrderType::LIMIT,
        }; // 0.20 buy - 3 qty (buy)

        // market order
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);

        let budget = dec!(2.1); // equivalent to 21
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        /*
         * 0.20 * 3 = 0.60
         * 0.25 * 10 = 2.50
         * Total = 3.10
         *
         * 2.1 -> 0.20 * 3 = 0.60
         * 2.1 - 0.60 = 1.50
         * 1.50 / 0.25 = 6
         * 6 * 0.25 = 1.50
         *
         * 6 + 3 = 9
         * (above calculation is done by `get_available_match_quantity` function)
         * So, we should match 3 orders at 0.20 and 6 orders at 0.25
         */

        assert_eq!(matches.len(), 2); // should match both orders
        assert_eq!(market_buy_order.filled_quantity, dec!(9));
        assert_eq!(market_buy_order.quantity, dec!(9)); // read above comment for calculation
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);
        assert_eq!(market_buy_order.price, budget);

        assert_eq!(matches[0].opposite_order_id, sell_order_2.id);
        assert_eq!(matches[0].matched_quantity, dec!(3)); // matched 3 orders at 0.20
        assert_eq!(matches[0].price, dec!(0.20)); // price of the matched order
        assert_eq!(matches[0].opposite_order_total_quantity, dec!(3)); // total quantity of the matched order
        assert_eq!(matches[0].opposite_order_filled_quantity, dec!(3)); // filled quantity of the matched order

        assert_eq!(matches[1].opposite_order_id, sell_order_1.id);
        assert_eq!(matches[1].matched_quantity, dec!(6)); // matched 6 orders at 0.25
        assert_eq!(matches[1].price, dec!(0.25)); // price of the matched order
        assert_eq!(matches[1].opposite_order_total_quantity, dec!(10)); // total quantity of the matched order
        assert_eq!(matches[1].opposite_order_filled_quantity, dec!(6)); // filled quantity of the matched order

        assert_eq!(outcome_book.bids.len(), 0); // no bids left
        assert_eq!(outcome_book.asks.len(), 1);
        assert!(outcome_book.asks.contains_key(&dec!(0.25))); // 0.25 ask left
        let price_level = outcome_book.asks.get(&dec!(0.25)).unwrap();

        assert_eq!(price_level.total_quantity, dec!(4)); // 10 - 6 = 4 left
        assert_eq!(price_level.orders.len(), 1); // only one order left at 0.25
        let order_book_entry = price_level.orders.get(0).unwrap();

        assert_eq!(order_book_entry.user_id, user_id);
        assert_eq!(order_book_entry.order_id, sell_order_1.id);
        assert_eq!(order_book_entry.filled_quantity, dec!(6)); // 6 filled
        assert_eq!(order_book_entry.total_quantity, dec!(10)); // total quantity is 10
    }

    #[test]
    fn test_add_order() {
        let price = Decimal::new(25, 2); // 0.25
        let created_at = get_created_at();
        let id = get_random_uuid();
        let market_id = get_random_uuid();
        let updated_at = get_created_at();
        let user_id = get_random_uuid();
        let order = Order {
            created_at,
            filled_quantity: Decimal::ZERO,
            id,
            market_id,
            outcome: Outcome::YES,         // yes side,
            price,                         // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::UNSPECIFIED,
            updated_at,
            user_id,
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&order);

        assert_eq!(outcome_book.bids.len(), 1);

        let price_level = outcome_book.bids.get(&price).unwrap();

        assert_eq!(price_level.total_quantity, Decimal::new(10, 0));
        assert_eq!(price_level.orders.len(), 1);

        let order_book_entry = price_level.orders.get(0).unwrap();

        assert_eq!(order_book_entry.user_id, user_id);
        assert_eq!(order_book_entry.order_id, id);
        assert_eq!(order_book_entry.filled_quantity, Decimal::ZERO);
        assert_eq!(order_book_entry.total_quantity, Decimal::new(10, 0));

        assert_eq!(outcome_book.best_bid(), Some(Decimal::new(25, 2)));
        assert_eq!(outcome_book.best_ask(), None);
    }

    #[test]
    fn test_remove_order() {
        let price = Decimal::new(25, 3);
        let mut order_book = OutcomeBook::default();

        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,         // yes side,
            price,                         // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::UNSPECIFIED,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),

            order_type: OrderType::LIMIT,
        };
        order_book.add_order(&order);
        let price_level = order_book.bids.get(&price).unwrap();

        assert_eq!(order_book.bids.len(), 1);
        assert_eq!(price_level.total_quantity, Decimal::new(10, 0));

        order_book.remove_order(order.id, OrderSide::BUY, price);

        let price_level = order_book.bids.get(&price);

        assert_eq!(order_book.bids.len(), 0);
        assert!(price_level.is_none());
    }

    #[test]
    fn test_update_order() {
        let price = Decimal::new(25, 2); // 0.25
        let created_at = get_created_at();
        let id = get_random_uuid();
        let market_id = get_random_uuid();
        let updated_at = get_created_at();
        let user_id = get_random_uuid();
        let quantity = Decimal::new(10, 0);
        let order = Order {
            created_at,
            filled_quantity: Decimal::ZERO,
            id,
            market_id,
            outcome: Outcome::YES,         // yes side,
            price,                         // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::UNSPECIFIED,
            updated_at,
            user_id,
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&order);

        assert_eq!(outcome_book.bids.len(), 1);

        let price_level = outcome_book.bids.get(&price).unwrap();
        assert_eq!(price_level.total_quantity, quantity);

        // updating order
        outcome_book._update_order_filled_quantity(id, OrderSide::BUY, price, Decimal::new(5, 0));

        let price_level = outcome_book.bids.get(&price).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(5, 0));
        let price_order = price_level.orders.get(0).unwrap();
        assert_eq!(price_order.filled_quantity, Decimal::new(5, 0));
    }

    #[test]
    fn test_match_order() {
        let market_id = get_random_uuid();

        let buy_order_1 = Order {
            order_type: OrderType::LIMIT,

            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),    // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        }; // 0.25 buy - 10 qty (buy)
        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),   // 0.25
            quantity: Decimal::new(3, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id: get_random_uuid(),
        }; // 0.20 buy - 3 qty (buy)
        let buy_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(23, 2),   // 0.25
            quantity: Decimal::new(4, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id: get_random_uuid(),
        }; // 0.23 buy - 4 qty (buy)

        let mut sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),
            quantity: Decimal::new(15, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id: get_random_uuid(),
        }; // 0.20 - 15 qty (sell)
        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let resp = outcome_book.match_order(&mut sell_order_1);
        // NEED TO PERFORM POST UPDATES ON ADDED ORDERS....
        let order_book_entry = outcome_book.bids.get(&dec!(0.20));
        assert!(order_book_entry.is_some());
        let order_book_entry = order_book_entry.unwrap();
        assert!(order_book_entry.orders.len() == 1);
        assert!(order_book_entry.orders[0].filled_quantity == dec!(1));

        assert_eq!(sell_order_1.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 3);

        // Verify matching happened in price-time priority order
        assert_eq!(resp[0].opposite_order_id, buy_order_1.id); // Best price (0.25) first
        assert_eq!(resp[1].opposite_order_id, buy_order_3.id); // Second best price (0.23)
        assert_eq!(resp[2].opposite_order_id, buy_order_2.id); // Third best price (0.20)
    }

    #[test]
    fn test_partial_fill() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),   // 0.25
            quantity: Decimal::new(5, 0), // 5
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),    // 0.20
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::new(5, 0));
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].matched_quantity, Decimal::new(5, 0)); // matched quantity
    }

    #[test]
    fn test_match_multiple_orders_same_price() {
        let market_id = get_random_uuid();
        let price = Decimal::new(25, 2); // 0.25

        // 3 buy orders at the same price
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(5, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(3, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(2, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(7, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 2); // Should match with the first two orders
        assert_eq!(resp[0].opposite_order_id, buy_order_1.id);
        assert_eq!(resp[1].opposite_order_id, buy_order_2.id);
    }

    #[test]
    fn test_no_matching_price() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2), // 0.20
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2), // 0.25 (higher than buy price)
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::ZERO);
        assert_eq!(resp.len(), 0);
    }

    #[test]
    fn test_large_order_book() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add 1000 buy orders at different prices
        for i in 1..=1000 {
            let buy_order = Order {
                created_at: get_created_at(),
                filled_quantity: Decimal::ZERO,
                id: get_random_uuid(),
                market_id,
                outcome: Outcome::YES,
                price: Decimal::new(i, 4), // range is 0.0001 to 1.0000
                quantity: Decimal::new(1, 0),
                side: OrderSide::BUY,
                status: OrderStatus::OPEN,
                updated_at: get_created_at(),
                order_type: OrderType::LIMIT,

                user_id,
            };
            outcome_book.add_order(&buy_order);
        }

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(5, 2),      // 0.05
            quantity: Decimal::new(500, 0), // Match with 500 highest bids
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 500);
        assert_eq!(sell_order.filled_quantity, Decimal::new(500, 0));
    }

    #[test]
    fn test_already_partially_filled_order() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2), // 0.25
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::new(5, 0), // Already filled 5 units
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2), // 0.20
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(sell_order.filled_quantity, Decimal::new(10, 0));
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].matched_quantity, Decimal::new(5, 0)); // Only needed to match 5 more
    }

    #[test]
    fn test_empty_order_book() {
        let market_id = get_random_uuid();

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::ZERO);
        assert_eq!(resp.len(), 0);
    }

    #[test]
    fn test_db_matching_order_issue() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let buy_order_one = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let buy_order_one_1 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let buy_order_one_2 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&buy_order_one);
        outcome_book.add_order(&buy_order_one_1);
        outcome_book.add_order(&buy_order_one_2);

        let mut matching_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let matches = outcome_book.match_order(&mut matching_sell_order);
        assert_eq!(matches.len(), 1);
        let price_level = outcome_book.bids.get(&dec!(0.61)).unwrap();
        assert_eq!(price_level.orders.len(), 2); // matched 1 order so 3 - 1 = 2
    }

    #[test]
    fn test_market_order_buy_with_zero_budget() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&sell_order);

        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        let matches = outcome_book.create_market_order(&mut market_buy_order, Decimal::ZERO);

        assert_eq!(matches.len(), 0);
        assert_eq!(market_buy_order.quantity, Decimal::ZERO);
        assert_eq!(market_buy_order.price, Decimal::ZERO);
        assert_eq!(market_buy_order.status, OrderStatus::CANCELLED);
    }

    #[test]
    fn test_market_order_sell_basic() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(75, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&buy_order);

        let mut market_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        let budget = dec!(7.5); // 0.75 * 10 = 7.5 units
        let matches = outcome_book.create_market_order(&mut market_sell_order, budget);

        assert_eq!(matches.len(), 1);
        assert_eq!(market_sell_order.filled_quantity, dec!(10));
        assert_eq!(market_sell_order.quantity, dec!(10));
        assert_eq!(market_sell_order.status, OrderStatus::FILLED);
        assert_eq!(market_sell_order.price, budget);
    }

    #[test]
    fn test_market_order_with_non_zero_price() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&sell_order);

        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(30, 2), // Non-zero price
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        let matches = outcome_book.create_market_order(&mut market_buy_order, dec!(2.5));

        assert_eq!(matches.len(), 0); // Should fail because market orders must have price 0
    }

    #[test]
    fn test_market_order_sell_multiple_price_levels() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        // Multiple buy orders at different price levels
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(80, 2), // 0.80
            quantity: Decimal::new(5, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(70, 2), // 0.70
            quantity: Decimal::new(3, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        let buy_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(60, 2), // 0.60
            quantity: Decimal::new(4, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let mut market_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        // Budget enough to fill all orders: 0.80*5 + 0.70*3 + 0.60*4 = 4.0 + 2.1 + 2.4 = 8.5
        let budget = dec!(8.5);
        let matches = outcome_book.create_market_order(&mut market_sell_order, budget);

        assert_eq!(matches.len(), 3);
        assert_eq!(market_sell_order.filled_quantity, dec!(12)); // 5 + 3 + 4 = 12
        assert_eq!(market_sell_order.quantity, dec!(12));
        assert_eq!(market_sell_order.status, OrderStatus::FILLED);

        // Matches should be in price priority (highest to lowest for sell orders)
        assert_eq!(matches[0].opposite_order_id, buy_order_1.id); // Highest price first
        assert_eq!(matches[0].price, dec!(0.80));
        assert_eq!(matches[1].opposite_order_id, buy_order_2.id);
        assert_eq!(matches[1].price, dec!(0.70));
        assert_eq!(matches[2].opposite_order_id, buy_order_3.id);
        assert_eq!(matches[2].price, dec!(0.60));
    }

    #[test]
    fn test_market_order_with_partial_budget() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(50, 2), // 0.50
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&buy_order);

        let mut market_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        // Budget for only 3 shares: 0.50 * 3 = 1.5
        let budget = dec!(1.5);
        let matches = outcome_book.create_market_order(&mut market_sell_order, budget);

        assert_eq!(matches.len(), 1);
        assert_eq!(market_sell_order.filled_quantity, dec!(3));
        assert_eq!(market_sell_order.quantity, dec!(3));
        assert_eq!(market_sell_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_market_order_with_fractional_quantity() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2), // 0.25
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&sell_order);

        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        // Budget for 1.25 shares: 0.25 * 1.25 = 0.3125
        let budget = dec!(0.3125);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(matches.len(), 1);
        assert_eq!(market_buy_order.filled_quantity, dec!(1.25));
        assert_eq!(market_buy_order.quantity, dec!(1.25));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_market_order_with_no_matching_orders() {
        let market_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        // Empty order book

        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        let budget = dec!(5.0);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(matches.len(), 0);
        assert_eq!(market_buy_order.quantity, Decimal::ZERO);
        assert_eq!(market_buy_order.status, OrderStatus::CANCELLED);
    }

    #[test]
    fn test_market_order_with_wrong_order_type() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&sell_order);

        let mut limit_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::LIMIT, // Wrong order type
        };

        let budget = dec!(2.5);
        let matches = outcome_book.create_market_order(&mut limit_buy_order, budget);

        assert_eq!(matches.len(), 0); // Should fail because only market orders can be used
    }

    #[test]
    fn test_get_available_match_quantity() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add multiple sell orders at different prices
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2), // 0.20
            quantity: Decimal::new(5, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(30, 2), // 0.30
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);

        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::MARKET,
        };

        // Test with different budgets
        // Budget enough for all sell_order_1: 0.20 * 5 = 1.0
        let quantity = outcome_book.get_available_match_quantity(&mut market_buy_order, dec!(1.0));
        assert_eq!(quantity, dec!(5.0));

        // Budget enough for all sell_order_1 and part of sell_order_2
        // 0.20 * 5 + 0.30 * 3.33 = 1.0 + 1.0 = 2.0
        let quantity = outcome_book.get_available_match_quantity(&mut market_buy_order, dec!(2.0));
        assert_eq!(quantity, dec!(5) + dec!(1.0) / dec!(0.30));

        // Budget enough for all orders: 0.20 * 5 + 0.30 * 10 = 1.0 + 3.0 = 4.0
        let quantity = outcome_book.get_available_match_quantity(&mut market_buy_order, dec!(4.0));
        assert_eq!(quantity, dec!(15.0)); // 5 + 10 = 15
    }

    #[test]
    fn test_market_order_with_self_match_prevention() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id, // Same user ID
        };

        outcome_book.add_order(&sell_order);

        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id, // Same user ID
            order_type: OrderType::MARKET,
        };

        let budget = dec!(2.5);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(matches.len(), 0); // Should not match with own orders
        assert_eq!(market_buy_order.quantity, Decimal::ZERO);
    }

    #[test]
    fn test_get_order_book() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add buy orders
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(70, 2), // 0.70
            quantity: Decimal::new(5, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(60, 2), // 0.60
            quantity: Decimal::new(3, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        // Add sell orders
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(80, 2), // 0.80
            quantity: Decimal::new(4, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(90, 2), // 0.90
            quantity: Decimal::new(2, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        };

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);

        let order_book = outcome_book.get_order_book();

        assert_eq!(order_book.bids.len(), 2);
        assert_eq!(order_book.asks.len(), 2);

        // Check bids
        let bid_levels: Vec<&OrderLevel> = order_book
            .bids
            .iter()
            .filter(|level| level.price == dec!(0.70) || level.price == dec!(0.60))
            .collect();

        assert_eq!(bid_levels.len(), 2);

        for level in bid_levels {
            if level.price == dec!(0.70) {
                assert_eq!(level.shares, dec!(5));
                assert_eq!(level.users, 1);
            } else if level.price == dec!(0.60) {
                assert_eq!(level.shares, dec!(3));
                assert_eq!(level.users, 1);
            }
        }

        // Check asks
        let ask_levels: Vec<&OrderLevel> = order_book
            .asks
            .iter()
            .filter(|level| level.price == dec!(0.80) || level.price == dec!(0.90))
            .collect();

        assert_eq!(ask_levels.len(), 2);

        for level in ask_levels {
            if level.price == dec!(0.80) {
                assert_eq!(level.shares, dec!(4));
                assert_eq!(level.users, 1);
            } else if level.price == dec!(0.90) {
                assert_eq!(level.shares, dec!(2));
                assert_eq!(level.users, 1);
            }
        }
    }
}

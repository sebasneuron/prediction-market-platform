use db_service::schema::{
    enums::{OrderSide, Outcome},
    orders::Order,
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::order_book::outcome_book::{OrderBookMatchedOutput, OutcomeBook};

use super::market_book::MarketBook;

#[derive(Debug)]
pub(crate) struct GlobalMarketBook {
    pub(crate) markets: HashMap<Uuid, MarketBook>,
}

impl GlobalMarketBook {
    pub(crate) fn new() -> Self {
        Self {
            markets: HashMap::new(),
        }
    }

    pub(crate) fn process_order(
        &mut self,
        order: &mut Order,
        liquidity_b: Decimal,
    ) -> Vec<OrderBookMatchedOutput> {
        let market_id = order.market_id;
        let market_book = self.get_or_create_market(market_id, liquidity_b);
        market_book.process_order(order)
    }

    pub(crate) fn process_order_without_liquidity(
        &mut self,
        order: &mut Order,
    ) -> Vec<OrderBookMatchedOutput> {
        let market_id = order.market_id;
        if let Some(market_book) = self.markets.get_mut(&market_id) {
            let matches = market_book.process_order(order);
            matches
        } else {
            Vec::new()
        }
    }

    pub(crate) fn add_order(&mut self, order: &Order, liquidity_b: Decimal) {
        let market_id = order.market_id;
        let market_book = self.get_or_create_market(market_id, liquidity_b);
        market_book.add_order(order);
    }

    pub(crate) fn get_market_price(&self, market_id: &Uuid, outcome: Outcome) -> Option<Decimal> {
        self.markets.get(market_id).map(|market| match outcome {
            Outcome::YES => market.current_yes_price,
            Outcome::NO => market.current_no_price,
            _ => Decimal::ZERO,
        })
    }

    pub(crate) fn get_orders(&self, market_id: &Uuid, outcome: Outcome) -> Option<&OutcomeBook> {
        self.markets
            .get(market_id)
            .and_then(|market| market.get_order_book(outcome))
    }

    pub(crate) fn create_market_order(
        &mut self,
        market_id: &Uuid,
        order: &mut Order,
        budget: Decimal,
    ) -> Vec<OrderBookMatchedOutput> {
        if let Some(market_book) = self.markets.get_mut(market_id) {
            market_book.create_market_order(order, budget)
        } else {
            Vec::new()
        }
    }

    pub(crate) fn remove_order(
        &mut self,
        market_id: Uuid,
        order_id: Uuid,
        market_side: OrderSide,
        outcome: Outcome,
        price: Decimal,
    ) -> bool {
        if let Some(market_book) = self.markets.get_mut(&market_id) {
            market_book.remove_order(order_id, market_side, outcome, price)
        } else {
            false
        }
    }

    pub(crate) fn update_order(
        &mut self,
        order: &mut Order,
        new_price: Decimal,
        new_total_quantity: Decimal,
    ) -> bool {
        if let Some(market_book) = self.markets.get_mut(&order.market_id) {
            market_book.update_order(order, new_total_quantity, new_price)
        } else {
            false
        }
    }

    pub(crate) fn remove_market(&mut self, market_id: &Uuid) -> bool {
        self.markets.remove(market_id).is_some()
    }

    fn get_or_create_market(&mut self, market_id: Uuid, liquidity_b: Decimal) -> &mut MarketBook {
        self.markets
            .entry(market_id)
            .or_insert(MarketBook::new(liquidity_b))
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;
    use db_service::schema::enums::{OrderSide, OrderStatus, OrderType, Outcome};
    use rust_decimal_macros::dec;

    use super::*;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_global_market_book_creation() {
        let mut global_market_book = GlobalMarketBook::new();

        assert_eq!(global_market_book.markets.len(), 0);

        let market_id = Uuid::new_v4();
        global_market_book
            .markets
            .insert(market_id, MarketBook::new(dec!(100)));

        assert_eq!(global_market_book.markets.len(), 1);

        let market_book = global_market_book.markets.get(&market_id);
        assert!(market_book.is_some());

        if let Some(book) = market_book {
            assert_eq!(book.current_no_price, dec!(0.5));
            assert_eq!(book.current_yes_price, dec!(0.5));
            assert_eq!(book.liquidity_b, dec!(100));
        } else {
            panic!("Market book should exist for the given market ID");
        }
    }

    #[test]
    fn test_process_order() {
        let mut global_market_book = GlobalMarketBook::new();
        let market_id = Uuid::new_v4();
        let liquidity_b = dec!(100);
        global_market_book
            .markets
            .insert(market_id, MarketBook::new(liquidity_b));

        let mut buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.5),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let results = global_market_book.process_order(&mut buy_order, liquidity_b);

        assert!(results.is_empty());

        // matching the order
        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.5),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let results = global_market_book.process_order(&mut sell_order, liquidity_b);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].order_id, sell_order.id);
        assert_eq!(results[0].opposite_order_id, buy_order.id);
        assert_eq!(results[0].matched_quantity, dec!(10)); // Matched quantity
        assert_eq!(results[0].price, dec!(0.5)); // Matched price
        assert_eq!(global_market_book.markets.len(), 1);
    }
}

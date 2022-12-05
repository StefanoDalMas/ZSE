use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::market::{Market, market_test};

use sha256::digest;
use unitn_market_2022::event::event::{Event, EventKind};

mod market;
mod wrapper;
mod tests;

#[derive(Hash)]
struct Request{
    good_kind: unitn_market_2022::good::good_kind::GoodKind,
    quantity: String,
    offer: String,
    name:String,
}
fn main() {
    println!("Henlo");
    let mut market = market::ZSE::new_random();
    let x = market.borrow_mut().lock_buy(unitn_market_2022::good::good_kind::GoodKind::USD,5.0,2.0,"prova".to_string());
    let _ = market.borrow_mut().buy(x.unwrap(), &mut unitn_market_2022::good::good::Good::new(unitn_market_2022::good::good_kind::GoodKind::EUR, 50000.0));
    let y = market.borrow_mut().lock_sell(unitn_market_2022::good::good_kind::GoodKind::USD,100.0,2.0,"prova".to_string());
    let _ = market.borrow_mut().sell(y.unwrap(), &mut unitn_market_2022::good::good::Good::new(unitn_market_2022::good::good_kind::GoodKind::USD, 102.0));
}


pub enum GoodKind {
    EUR,
    YEN,
    USD,
    YUAN,
}
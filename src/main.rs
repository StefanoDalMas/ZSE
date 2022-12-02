use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::market::Market;

use sha256::digest;
use unitn_market_2022::event::event::{Event, EventKind};

mod market;
mod wrapper;
mod tests;

#[derive(Hash)]
struct request{
    good_kind: unitn_market_2022::good::good_kind::GoodKind,
    quantity: String,
    offer: String,
    name:String,
}
fn main() {
    println!("Henlo");
    let mut market = market::ZSE::new_random();
}


pub enum GoodKind {
    EUR,
    YEN,
    USD,
    YUAN,
}
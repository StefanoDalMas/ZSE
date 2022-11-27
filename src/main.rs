use std::borrow::{Borrow, BorrowMut};
use unitn_market_2022::market::Market;

mod market;
mod wrapper;

fn main() {
    println!("Hello, world!");
    println!("Henlo");
    let mut market = market::ZSE::new_random();
}
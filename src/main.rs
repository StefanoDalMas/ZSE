use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::market::{Market, market_test};

use sha256::digest;
use unitn_market_2022::event::event::{Event, EventKind};

pub mod market;
mod trader;
mod coolvisualizer;
mod filereader;
mod trader_balordo;
mod common;


#[derive(Hash)]
struct Request_good{
    good_kind: unitn_market_2022::good::good_kind::GoodKind,
    quantity: String,
    offer: String,
    name:String,
}

fn main() {
    /* trader marina
    let mut trader = trader::ZSE_Trader::new();
    println!("{}", trader.get_name());
    for market in trader.get_markets() {
        println!("\n{}", market.borrow().get_name());
        println!("{}", market.borrow().get_budget());
        println!("{:?}", market.borrow().get_goods());
    }

    trader.update_all_prices();
    trader.print_prices();
    println!();
    let mut count = 0;
    let mut state = true;
    //while trader.get_qty_euro_trader() > 50.0 { //num messo a caso 
    while state{
        state = trader.strat1(count);
        count += 1;
        // if trader.count() == 10 { break; }
    }
    println!();
    trader.print_goods_trader();
    trader.print_data();
    println!("tot cicli: {}", count);
    */

    //trader andy
    //let mut trader = trader_balordo::ZSE_Trader::new();
    //trader.trade();
    //VISUALIZER STUFF DON'T TOUCH PLZ
    //coolvisualizer::try_viz();
}

pub enum GK {
    EUR,
    YEN,
    USD,
    YUAN,
}

#[derive(Copy, Clone)]
enum Mode {
    Buy,
    Sell,
}
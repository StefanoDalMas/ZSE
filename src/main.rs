use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
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
    // trader marina
    let mut trader = trader::ZSE_Trader::new();
    
    let mut count = 0;
    let mut state = true;
     while state{
        state = trader.strategy(count);
        count += 1;
    }
    println!();
    println!();
    trader.print_goods_trader();
    trader.print_data();
    println!("tot cicli: {}", count);

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
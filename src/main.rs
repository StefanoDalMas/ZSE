use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::market::{Market, market_test};

use sha256::digest;
use unitn_market_2022::event::event::{Event, EventKind};

pub mod market;
mod trader;

#[derive(Hash)]
struct Request_good{
    good_kind: unitn_market_2022::good::good_kind::GoodKind,
    quantity: String,
    offer: String,
    name:String,
}

fn main() {
    /* 
    println!("Init market");
    let mut market = market::ZSE::new_random();
    println!("{}", market.borrow().get_budget());
    println!("{:?}", market.borrow().get_goods());

    println!("Lock buy");
    let x = market.borrow_mut().lock_buy(unitn_market_2022::good::good_kind::GoodKind::USD,5.0,7.0,"test".to_string());
    println!("{}", market.borrow().get_budget());
    println!("{:?}", market.borrow().get_goods());

    println!("Buy");
    let _ = market.borrow_mut().buy(x.unwrap(), &mut unitn_market_2022::good::good::Good::new(unitn_market_2022::good::good_kind::GoodKind::EUR, 50000.0));
    println!("{}", market.borrow().get_budget());
    println!("{:?}", market.borrow().get_goods());

    println!("Lock sell and sell");
    let y = market.borrow_mut().lock_sell(unitn_market_2022::good::good_kind::GoodKind::USD,100.0,2.0,"test".to_string());
    let _ = market.borrow_mut().sell(y.unwrap(), &mut unitn_market_2022::good::good::Good::new(unitn_market_2022::good::good_kind::GoodKind::USD, 102.0));
    println!("{}", market.borrow().get_budget());
    println!("{:?}", market.borrow().get_goods());
    */
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
use std::cell::RefCell;
use std::rc::Rc;
use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::market::Market;

pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
}

impl ZSE_Trader {
    pub fn new() -> Self {
        let name = "ZSE_Trader".to_string();
        let mut markets = Vec::new();
        markets.push(RCNZ::new_random());
        markets.push(Bfb::new_random());
        markets.push(BVCMarket::new_random());
        Self { name, markets }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_markets(&self) -> &Vec<Rc<RefCell<dyn Market>>> {
        &self.markets
    }
}
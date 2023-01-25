use std::cell::RefCell;
use std::rc::Rc;

use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;

pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>,
}

enum Mode {
    Buy,
    Sell,
}

enum Bazaar {
    RCNZ,
    Bfb,
    BVC,
}

impl ZSE_Trader {
    pub fn new() -> Self {
        let name = "ZSE_Trader".to_string();
        let mut markets = Vec::new();
        markets.push(RCNZ::new_random());
        markets.push(Bfb::new_random());
        markets.push(BVCMarket::new_random());
        let prices = vec![vec![vec![0.0; 4]; 3]; 2];
        Self { name, markets, prices }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_markets(&self) -> &Vec<Rc<RefCell<dyn Market>>> {
        &self.markets
    }

    pub fn get_prices(&self) -> &Vec<Vec<Vec<f32>>> {
        &self.prices
    }

    pub fn update_all_prices(&mut self) {
        for m in &self.markets {
            let index = get_index_by_market(m.borrow().get_name());
            let goods = m.borrow().get_goods();
            for g in goods {
                let index_kind = get_index_by_goodkind(&g.good_kind);
                self.prices[0][index][index_kind] = g.exchange_rate_buy;
                self.prices[1][index][index_kind] = g.exchange_rate_sell;
            }
        }
    }

    pub fn print_prices(&self) {
        for i in 0..self.prices.len() {
            if i == 0 {
                println!("\nBuy prices:");
            } else {
                println!("\nSell prices:");
            }
            println!("      EUR      USD         YEN          YUAN");
            for j in 0..self.prices[i].len() {
                let name = match j {
                    0 => "RCNZ",
                    1 => "BFB",
                    2 => "BVC",
                    _ => panic!("Error in print_prices"),
                };
                print!("{}:\t", name);
                for k in 0..self.prices[i][j].len() {
                    print!("{}\t", self.prices[i][j][k]);
                }
                println!();
            }
        }
    }
}

fn get_index_by_market(m: &str) -> usize {
    match m {
        "RCNZ" => 0,
        "Baku stock exchange" => 1,
        "BVC" => 2,
        _ => panic!("Market not found"),
    }
}

fn get_index_by_goodkind(kind: &GoodKind) -> usize {
    return match *kind {
        GoodKind::EUR => 0,
        GoodKind::USD => 1,
        GoodKind::YEN => 2,
        GoodKind::YUAN => 3,
    };
}
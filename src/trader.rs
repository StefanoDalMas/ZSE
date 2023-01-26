use std::cell::RefCell;
use std::fmt::Debug;
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
#[derive(Debug,Clone)]
enum Mode {
    Buy,
    Sell,
}

enum Bazaar {
    RCNZ,
    Bfb,
    BVC,
}

#[derive(Debug, Clone)]
pub struct Value{
    val: f32,
    market: String,
    mode: Mode, //mode of price in the market (sell_price -> trader want to buy some goods and vice-versa)
}

impl Value{
    fn new_buy() -> Self{
        Value { val: 1000000.0, market: "".to_string(), mode: Mode::Buy }
    }
    fn new_sell() -> Self{
        Value { val: 1000000.0, market: "".to_string(), mode: Mode::Sell }
    }
}
impl PartialEq for Value{
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}
impl PartialOrd for Value{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.val.partial_cmp(&other.val)
    }
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
                let name = get_name_market(j);
                print!("{}:\t", name);
                for k in 0..self.prices[i][j].len() {
                    print!("{}\t", self.prices[i][j][k]);
                }
                println!();
            }
        }
    }

    pub fn find_min_sell_price(&self){ //in order to buy with less loss
        use rand::Rng;
        let mut min_sell_price_market: Vec<Value> = vec![Value::new_sell(); 4];
        for g in 0..4{ //goodking
            for i in 0..self.prices[0].len(){ //market
                if min_sell_price_market[g].val > self.prices[0][i][g]{
                    min_sell_price_market[g].val = self.prices[0][i][g];
                    min_sell_price_market[g].market = get_name_market(i);
                } else if min_sell_price_market[g].val == self.prices[0][i][g]{
                    let num = rand::thread_rng().gen_range(0..100);
                    if num % 2 == 0 {
                        min_sell_price_market[g].val = self.prices[0][i][g];
                        min_sell_price_market[g].market = get_name_market(i);
                    }
                }
            }
        }
        for g in 0..4{
            println!("{} - {:?}", get_goodkind(g), min_sell_price_market[g]);
        }
    }
    
    pub fn find_mid_buy_price(&self){
        let mut mid_buy_price_market: Vec<Value> = vec![Value::new_sell(); 4];
        for g in 0..4{
            let min_buy = self.find_min_max_buy(g).0;
            let max_buy = self.find_min_max_buy(g).1;
            let mut vec = vec![0,1,2];
            vec.remove(min_buy);
            vec.remove(max_buy);
            mid_buy_price_market[g].val = self.prices[1][vec[0]][g];
            mid_buy_price_market[g].market = get_name_market(vec[0]);
        }
        for g in 0..4{
            println!("{} - {:?}", get_goodkind(g), mid_buy_price_market[g]);
        }
    }
    
    pub fn find_min_max_buy(&self, g: usize) -> (usize, usize){
        let mut min = 10000.0;
        let mut max = 0.0;
        let mut x = 3;
        let mut y = 3;
        for i in 0..self.prices[0].len(){ //market
            if min > self.prices[0][i][g]{
                min = self.prices[0][i][g];
                x = i;
            }
            if max < self.prices[0][i][g]{
                max = self.prices[0][i][g];
                y = i;
            }
        }
        (x, y) //(min, max)
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

fn get_name_market(n: usize) -> String{
    let name = match n {
        0 => "RCNZ".to_string(),
        1 => "BFB".to_string(),
        2 => "BVC".to_string(),
        _ => panic!("Error in print_prices"),
    };
    name
}

fn get_index_by_goodkind(kind: &GoodKind) -> usize {
    return match *kind {
        GoodKind::EUR => 0,
        GoodKind::USD => 1,
        GoodKind::YEN => 2,
        GoodKind::YUAN => 3,
    };
}

fn get_goodkind(i: usize) -> String{
    let gk = match i {
        0 => "EUR",
        1 => "USD",
        2 => "YEN",
        3 => "YUAN",
        _ => "",
    };
    gk.to_string()
}
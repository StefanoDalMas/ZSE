use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use rand::Rng;
use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{Market, LockBuyError, LockSellError, BuyError};
use unitn_market_2022::subscribe_each_other;

const STARTING_CAPITAL: f32 = 10000000.0; //decidere noi, messa a caso
pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>, //prices of markets
    goods: Vec<Good>, //goods of the trader
    //vec di goods e penso anche dei prezzi per fare cose
}
#[derive(Debug,Clone)]
enum Mode {
    Buy, //0
    Sell, //1
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
        Value { val: 0.0, market: "".to_string(), mode: Mode::Buy }
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
        subscribe_each_other!(markets[0], markets[1], markets[2]);
        let prices = vec![vec![vec![0.0; 4]; 3]; 2];
        let goods = vec![
            Good::new(GoodKind::EUR, STARTING_CAPITAL), 
            Good::new(GoodKind::USD, 0.0), 
            Good::new(GoodKind::YEN, 0.0), 
            Good::new(GoodKind::YUAN, 0.0),
        ];
        
        Self { name, markets, prices , goods }
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
            let index = get_index_by_market(m.borrow_mut().get_name());
            let goods = m.borrow_mut().get_goods();
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

    fn find_min_sell_price(&self) -> Vec<Value>{ 
        let mut min_sell_price_market: Vec<Value> = vec![Value::new_sell(); 4];
        for g in 0..4{ //goodking
            for i in 0..self.prices[1].len(){ //market
                if min_sell_price_market[g].val > self.prices[1][i][g]{
                    min_sell_price_market[g].val = self.prices[1][i][g];
                    min_sell_price_market[g].market = get_name_market(i);
                } else if min_sell_price_market[g].val == self.prices[1][i][g]{
                    let num = rand::thread_rng().gen_range(0..100);
                    if num % 2 == 0 {
                        min_sell_price_market[g].val = self.prices[1][i][g];
                        min_sell_price_market[g].market = get_name_market(i);
                    }
                }
            }
        }
        min_sell_price_market
    }
    
    fn find_mid_buy_price(&self) -> Vec<Value>{
        let mut mid_buy_price_market: Vec<Value> = vec![Value::new_buy(); 4];
        for g in 0..4{
            let min_buy = self.find_min_max_buy(g).0;
            let max_buy = self.find_min_max_buy(g).1;
            let mut vec = vec![0,1,2];
            vec.remove(min_buy);
            vec.remove(max_buy);
            mid_buy_price_market[g].val = self.prices[0][vec[0]][g];
            mid_buy_price_market[g].market = get_name_market(vec[0]);
        }
        mid_buy_price_market
    }
    
    fn find_min_max_buy(&self, g: usize) -> (usize, usize){
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

    pub fn try_lock_and_buy(&mut self) { 
        let want_buy = self.find_mid_buy_price();
        let index = rand::thread_rng().gen_range(1..4);
        let g = get_goodkind_by_index(&index);
        let prova = &want_buy[index].market;
        println!("{} -> {:?}", g, want_buy[index]);
        
        let m = &self.markets[get_index_by_market(&prova)];
        let string: Result<String, LockBuyError>;
        let offer: f32;
        let qty = 20.0;
        let b: Result<Good, BuyError>;
        
        let min_bid_offer = m.borrow_mut().get_buy_price(g, 20.0);
        if min_bid_offer.is_ok(){
            offer = (min_bid_offer.unwrap() as i32 ) as f32 + 0.82 ;
            string = m.borrow_mut().lock_buy(g, qty, offer, self.get_name().clone());
            if let Ok(token) = string {
                b = m.borrow_mut().buy(token, &mut self.goods[0]);
                let res = self.goods[index].merge(Good::new(g, qty));
                if res.is_err(){ panic!("Error: {:?}", res); }
                println!("buy {:?} : {:?}", g, b);
                println!("{} -- {}", self.goods[0], self.goods[index]);
                self.update_all_prices(); 
                self.print_prices();      
            }
        } else { panic!("Market error"); }
    }
    
    pub fn print_goods(&self){
        for g in &self.goods{
            println!("{:?} ", g );
        }
    }

}


fn get_index_by_market(m: &str) -> usize {
    match m {
        "RCNZ" => 0,
        "Baku stock exchange" => 1,
        "BFB" => 1,
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

fn get_goodkind_by_index(i: &usize) -> GoodKind{
    return match *i {
        0 => GoodKind::EUR,
        1 => GoodKind::USD,
        2 => GoodKind::YEN,
        _ => GoodKind::YUAN,
    };
}
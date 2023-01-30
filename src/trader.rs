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

const STARTING_CAPITAL: f32 = 100000.0; //decidere noi, messa a caso
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
}

impl Value{
    fn new_max() -> Self{
        Value { val: 0.0, market: "".to_string() }
    }
    fn new_min() -> Self{
        Value { val: 1000000.0, market: "".to_string() }
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
            println!("\tEUR\tUSD\t\tYEN\t\tYUAN");
            for j in 0..self.prices[i].len() {
                let name = get_name_market(j);
                print!("{}:\t", name);
                for k in 0..self.prices[i][j].len() {
                    print!("{}\t", self.prices[i][j][k]);
                }
                println!();
            }
            println!();
        }
    }

    pub fn print_goods_trader(&self){
        for g in &self.goods{
            println!("{:?} ", g );
        }
    }
    
    pub fn strategy_1(&mut self, x: i32){
        try_lock_and_buy(self, x);
    }
   
    pub fn get_qty_euro_trader(&mut self) -> f32{
        self.goods[0].get_qty()
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

fn find_min_price(t: &mut ZSE_Trader, mode: Mode, gk: usize) -> Value{
    let mut price_market: Value = Value::new_min();
    let x = match mode {
        Mode::Buy => 0,
        Mode::Sell => 1,
    };
    for i in 0..t.prices[x].len(){ //market
        if t.prices[x][i][gk]>0.0  && price_market.val > t.prices[x][i][gk]{ //nel mentre che RCNZ fixa i prezzi neg
            price_market.val = t.prices[x][i][gk];
            price_market.market = get_name_market(i);
        } else if price_market.val == t.prices[x][i][gk]{
            let num = rand::thread_rng().gen_range(0..100);
            if num % 2 == 0 {
                price_market.val = t.prices[x][i][gk];
                price_market.market = get_name_market(i);
            }
        }
    }
    price_market
}

fn find_max_price(t: &mut ZSE_Trader, mode: Mode, gk: usize) -> Value{
    let mut price_market: Value = Value::new_max();
    let x = match mode {
        Mode::Buy => 0,
        Mode::Sell => 1,
    };
    for i in 0..t.prices[x].len(){ //market
        if price_market.val < t.prices[x][i][gk]{
            price_market.val = t.prices[x][i][gk];
            price_market.market = get_name_market(i);
        } else if price_market.val == t.prices[x][i][gk]{
            let num = rand::thread_rng().gen_range(0..100);
            if num % 2 == 1 {
                price_market.val = t.prices[x][i][gk];
                price_market.market = get_name_market(i);
            }
        }
    }
    price_market
}

fn find_mid_price(t: &mut ZSE_Trader, mode: Mode, gk: usize) -> Value {
    let mut price_market: Value = Value::new_max();
    let min = get_index_by_market((find_min_price(t, mode.clone(), gk).market).as_str());
    let max = get_index_by_market((find_max_price(t, mode.clone(), gk).market).as_str());
    println!("min: {} - max: {}", min, max);
    
    let mut v: Vec<usize>= vec![0,1,2];
    v.retain(|&x| x!=min && x!=max);
    
    let x = match mode {
        Mode::Buy => 0,
        Mode::Sell => 1,
    };

    if t.prices[x][v[0]][gk] < 0.0 {
        v[0] = min;
    }
    
    price_market.market = get_name_market(v[0]);
    price_market.val = t.prices[x][v[0]][gk];
    price_market
}

fn try_lock_and_buy(trader: &mut ZSE_Trader, count: i32) { 
    let index = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
    let g = get_goodkind_by_index(&index);

    /*let mut method = || -> Vec<Value>{
        if count%2==0{ find_min_price(self, Mode::Buy, index) }
        else if count%3==1 { find_max_price(self, Mode::Buy, index) } 
        else { find_mid_price(self, Mode::Buy, index) }
    }; */ 
    //let want_buy = method();
    let want_buy = if count%3==0{ find_min_price(trader, Mode::Buy, index) }
                                else if count%3==1 { find_max_price(trader, Mode::Buy, index) } 
                                else { find_mid_price(trader, Mode::Buy, index) };
    //let want_buy = Value{ val: self.prices[0][2][index], market: "BVC".to_string() };
    let prova = &want_buy.market;
    println!("{} -> {:?} [{}]", g, want_buy, count%3);
    
    let m = &trader.markets[get_index_by_market(&prova)];
    let string: Result<String, LockBuyError>;
    let offer: f32;
    let qty = 20.0;
    let b: Result<Good, BuyError>;
    
    let min_bid_offer = m.borrow_mut().get_buy_price(g, 20.0);
    
    if min_bid_offer.is_ok(){
        offer = min_bid_offer.clone().unwrap() + 0.8293 ;
        println!("PROVA: {} vs {}", min_bid_offer.unwrap().clone(), offer);
        string = m.borrow_mut().lock_buy(g, qty, offer, trader.get_name().clone());
        if let Ok(token) = string {
            b = m.borrow_mut().buy(token, &mut trader.goods[0]);
            let res = trader.goods[index].merge(Good::new(g, qty));
            if res.is_err(){ panic!("Error: {:?}", res); }
            println!("buy {:?} : {:?}", g, b);
            println!("{} -- {}", trader.goods[0], trader.goods[index]);
            trader.update_all_prices(); 
            trader.print_prices();      
        } else { panic!("{:?}", string); }
    } else { panic!("Market error"); }
}
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use rand::Rng;
use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{Market, LockBuyError, LockSellError, BuyError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};

const STARTING_CAPITAL: f32 = 100000.0; //decidere noi, messa a caso
pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>, //prices of markets
    goods: Vec<Good>, //goods of the trader
    token_buy: Vec<(String, Rc<RefCell<dyn Market>>, GoodKind, f32, f32)>,
    token_sell: Vec<(String, Rc<RefCell<dyn Market>>, GoodKind, f32, f32)>,
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
        let token_buy = Vec::new();
        let token_sell = Vec::new();
        Self { name, markets, prices, goods, token_buy, token_sell }
    }

    pub fn get_name(&self) -> &String { &self.name }

    pub fn get_markets(&self) -> &Vec<Rc<RefCell<dyn Market>>> { &self.markets }

    pub fn len_token_buy(&self) -> usize { self.token_buy.len() }

    pub fn get_prices(&self) -> &Vec<Vec<Vec<f32>>> { &self.prices }

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
    
    pub fn strat_1(&mut self, x: i32){
        let index_gk_buy = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
        let gk_buy = get_goodkind_by_index(&index_gk_buy);
        
        let want_buy = choose(self, x, index_gk_buy, Mode::Buy);
        
        let mb = &self.markets[get_index_by_market(&want_buy.market)].clone();
        let qty_to_buy =  generate_qty(mb, gk_buy);

        if self.len_token_buy()>1 { //wait 'til we have 2 lock_buy and then buy both
            while self.len_token_buy() > 0 {
                try_buy(self); 
            }
        } else { 
            let wait = try_lock_buy(self, mb, gk_buy, qty_to_buy);
            if wait == false{  wait_one_day!(self.get_markets()[0], self.get_markets()[1], self.get_markets()[2]); }
        }
        
        let index_gk_sell = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
        let gk_sell = get_goodkind_by_index(&index_gk_sell);
        let want_sell = choose(self, x, index_gk_sell, Mode::Sell);
        let ms = &self.markets[get_index_by_market(&want_sell.market)].clone();
        //let qty_sell = ge

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
    
    let mut v: Vec<usize>= vec![0,1,2];
    v.retain(|&x| x!=min && x!=max);
    
    let x = match mode {
        Mode::Buy => 0,
        Mode::Sell => 1,
    };

    if t.prices[x][v[0]][gk] < 0.0 { v[0] = min; }
    
    price_market.market = get_name_market(v[0]);
    price_market.val = t.prices[x][v[0]][gk];
    price_market
}

fn choose(trader: &mut ZSE_Trader, count: i32, index: usize, mode: Mode) -> Value{
    if count%3==0{ find_min_price(trader, mode, index) }
    else if count%3==1 { find_max_price(trader, mode, index) } 
    else { find_mid_price(trader, mode, index) }
}

fn try_lock_buy(trader: &mut ZSE_Trader, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, qty: f32) -> bool{ 
    let string: Result<String, LockBuyError>; //token
    let offer: f32;
    
    if qty > 0.0{
        let min_bid_offer = market.borrow_mut().get_buy_price(gk, qty);
        if min_bid_offer.is_ok(){  
            offer = min_bid_offer.clone().unwrap() + 0.8293 ;
            let last_lock = trader.len_token_buy();
            if offer > trader.goods[0].get_qty() { //check to prevent InsufficientGoodQuantity - buy
                return false;  //CANNOT AFFORD
            }
            if last_lock>0 && offer > trader.token_buy[last_lock-1].4{ //seeing that i want to do 2 lock and buy in the future
                return false; //CANNOT AFFORD
            }
            string = market.borrow_mut().lock_buy(gk, qty, offer, trader.get_name().clone());
            if let Ok(token) = string { 
                println!("want to buy: {} -> {:?}", gk, market.borrow_mut().get_name());
                let prob_qty_euro = trader.goods[0].get_qty() - offer;
                println!("offer: {}, prima: {}, dopo: {}", offer, trader.goods[0].get_qty(), prob_qty_euro);
                trader.token_buy.push((token, market.clone(), gk, qty, prob_qty_euro)); 
            } else { panic!("{:?}", string); }
        } else { panic!("Market error: {:?}", min_bid_offer); }
        true
    } else {
        false
    }
}

fn try_buy(trader: &mut ZSE_Trader){
    let token = trader.token_buy[0].0.clone();
    let market = trader.token_buy[0].1.clone();
    let gk = trader.token_buy[0].2;
    let qty = trader.token_buy[0].3;

    let buy = market.borrow_mut().buy(token, &mut trader.goods[0]);
    if buy.is_err(){ panic!("Buy Error: {:?}", buy); }
    let res = trader.goods[get_index_by_goodkind(&gk)].merge(Good::new(gk, qty));
    if res.is_err(){ panic!("Merge Error: {:?}", res); }
                
    print!("buy {:?} with {}: {:?}\t", gk, market.borrow_mut().get_name(), buy);
    println!("{} -- {}\n", trader.goods[0], trader.goods[get_index_by_goodkind(&gk)]);
    trader.update_all_prices(); 
    trader.token_buy.remove(0);
}

fn generate_qty(market: &Rc<RefCell<dyn Market>>, gk: GoodKind) -> f32{ 
    let mut qty = rand::thread_rng().gen_range(1.0 ..200.0);
    let check = market.borrow_mut().get_goods();
    for x in check.iter(){ // check to prevent InsufficientGoodQuantityAvailable - lockbuy
        if x.good_kind == gk && x.quantity < qty { qty = x.quantity; }
    }
    qty
}
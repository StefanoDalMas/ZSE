use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use rand::Rng;
use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{Market, LockBuyError, LockSellError, BuyError, MarketGetterError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};


const STARTING_CAPITAL: f32 = 100000.0; //decidere noi, messa a caso
pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>, //prices of markets
    goods: Vec<Good>, //goods of the trader
    token_buy: Vec<Locking>,
    token_sell: Vec<Locking>,
    information: Data,
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
pub struct Locking{ //keep info about tokens and locks that trader does
    token: String,
    market: Rc<RefCell<dyn Market>>,
    time: i32,
    kind: GoodKind,
    qty: f32,
    new_qty_euro: f32,
    new_qty_gk: f32,
}
impl Display for Locking{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} - {}", self.token, self.market.borrow_mut().get_name(), self.time)
    }
}

#[derive(Debug, Clone, Copy)]
struct Data{ //general info
    lock_buy: i32,
    lock_sell: i32,
    buy: i32,
    sell: i32,
    wait: i32,
}
impl Data{
    fn new() -> Data{ Data{ lock_buy: 0, lock_sell: 0, buy: 0, sell: 0, wait: 0 } }
}

#[derive(Debug, Clone)]
pub struct Value{ //finding price
    val: f32,
    market: usize,
}
impl Value{
    fn new_max() -> Self{
        Value { val: 0.0, market: 3 }
    }
    fn new_min() -> Self{
        Value { val: 1000000.0, market: 3 }
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
        let information = Data::new();
        Self { name, markets, prices, goods, token_buy, token_sell, information }
    }

    pub fn get_name(&self) -> &String { &self.name }

    pub fn get_markets(&self) -> &Vec<Rc<RefCell<dyn Market>>> { &self.markets }

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
        println!();
    }

    pub fn print_data(&self){ println!("{:?}", self.information); }

    pub fn count(&self) -> i32{ 
        self.information.buy +
        self.information.lock_buy +
        self.information.sell +
        self.information.lock_sell +
        self.information.wait 
    }

    pub fn get_qty_good_trader(&mut self, i: usize) -> f32{ self.goods[i].get_qty() }

    pub fn strat1(&mut self, x: i32) -> bool{
        let mut lock: bool;
        let mut done = 0;
        if self.get_qty_good_trader(0) > STARTING_CAPITAL/2.0 {
            //BUY
            let index_gk_buy = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
            let gk_buy = get_goodkind_by_index(&index_gk_buy);
            let mut count_lock_buy = 0;
            

            let want_buy = self.chose(x, index_gk_buy, Mode::Buy); //chose between finding min/max/mid price
            let mb = &self.markets[want_buy.market].clone();
            let qty_to_buy =  self.generate_qty(mb, gk_buy, Mode::Buy);

            if mb.borrow_mut().get_name() == "BVC" {
                for i in self.token_buy.iter(){
                    if i.to_owned().market.borrow_mut().get_name() == "BVC" { count_lock_buy += 1; }
                }
            }
            if count_lock_buy < 4 { lock = self.try_lock_buy(mb, gk_buy, qty_to_buy); } else { lock = false; }
            if lock { self.information.lock_buy += 1; } 
            else {
                wait_one_day!(self.markets[0], self.markets[1], self.markets[2]); 
                self.information.wait += 1;
                println!("\nWAITING LOCK-BUY\n");
            }
            self.update_time();
            self.update_all_prices();
            if x % 3 == 2 {
                while self.token_buy.len()>0 {     
                    if self.try_buy(){ self.information.buy += 1; }
                    else {
                        wait_one_day!(self.markets[0], self.markets[1], self.markets[2]); 
                        self.information.wait += 1;
                        println!("\nWAITING BUY\n");
                    }
                    self.update_time();
                    self.update_all_prices();
                }
            }
            done = 1;
        }
        if self.get_qty_good_trader(1) > 200.0 || self.get_qty_good_trader(2) > 200.0 || self.get_qty_good_trader(3) > 200.0 {
            //SELL
            let index_gk_sell = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
            let gk_sell = get_goodkind_by_index(&index_gk_sell);
            let mut count_lock_sell = 0;
    
            let want_sell = self.chose(x, index_gk_sell, Mode::Sell);
            let ms = &self.markets[want_sell.market].clone();
            let qty_sell = self.generate_qty(ms, gk_sell, Mode::Sell); 
            
            if ms.borrow_mut().get_name() == "BVC" {
                for i in self.token_sell.iter(){
                    if i.to_owned().market.borrow_mut().get_name() == "BVC" { count_lock_sell += 1; }
                }
            }
            if count_lock_sell < 4 { lock = self.try_lock_sell(ms, gk_sell, qty_sell); } else { lock = false; }
            if lock {  self.information.lock_sell += 1; } 
            else {
                wait_one_day!(self.markets[0], self.markets[1], self.markets[2]); 
                self.information.wait += 1;
                println!("\nWAITING LOCK-SELL\n");
            }
            self.update_time();
            self.update_all_prices();
            //if x % 2 == 0 && self.token_sell.len()>0 { //TODO -> fare piu lock sell
            while self.token_sell.len()>0 {     
                if self.try_sell(){ self.information.sell += 1; }
                else {
                    wait_one_day!(self.markets[0], self.markets[1], self.markets[2]); 
                    self.information.wait += 1;
                    println!("\nWAITING SELL\n");
                }
                self.update_time();
                self.update_all_prices();
            }
            //}
            done = 2;
        }
        if done == 1 || done == 2 { true } else { false }

        /*
        let mut jump = false;
        let mut lock: bool;

        let index_gk_buy = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
        let gk_buy = get_goodkind_by_index(&index_gk_buy);
        
        let want_buy = self.choose(x, index_gk_buy, Mode::Buy);
        let mb = &self.markets[get_index_by_market(&want_buy.market)].clone();
        let qty_to_buy =  self.generate_qty(mb, gk_buy, Mode::Buy);
        
        if qty_to_buy == 0.0 { jump = true; }
        if self.token_buy.len()<2 && jump == false{ //wait 'til we have 2 lock_buy and then buy both
            lock = self.try_lock_buy(mb, gk_buy, qty_to_buy); 
            // if lock == false{ self.wait(); } 
            // else { 
            if lock{
                self.update_time();
                self.information.lock_buy += 1;
            }
            self.update_all_prices();
        } 
        if self.token_buy.len()>1 && jump == false { 
            while self.token_buy.len() > 0 {
                self.try_buy();
                self.information.buy += 1;
                self.update_time();
            }
        }
        jump = true;
        if self.information.buy>0 && self.information.buy%2==0 && jump { //lock_sell after 2 lock_buy and so buy
            let mut index_gk_sell = rand::thread_rng().gen_range(0..18)%3+1; //chose randomly between USD, YEN, YUAN 
            index_gk_sell = self.check_gk_sell(index_gk_sell); //make sure i can sell that gk 
            let gk_sell = get_goodkind_by_index(&index_gk_sell);
            let mut count_lock_sell = 0;

            let want_sell = self.choose(x, index_gk_sell, Mode::Sell);
            let ms = &self.markets[get_index_by_market(&want_sell.market)].clone();
            let qty_sell = self.generate_qty(ms, gk_sell, Mode::Sell); 
            if qty_sell > 0.0 { 
                if ms.borrow_mut().get_name() == "BVC"{
                    // for x in self.token_sell.iter(){
                    //     if x.market.as_ref().borrow_mut().get_name() == "BVC"{
                    //         count_lock_sell +=1;
                    //     }
                    // }
                    let _ = self.token_sell.iter().map(|x| {
                        if x.market.as_ref().borrow_mut().get_name() == "BVC"{ count_lock_sell +=1 }
                    });
                    println!("LOCK: {}", count_lock_sell);
                }
                if count_lock_sell < 4{
                    lock = self.try_lock_sell(ms, gk_sell, qty_sell);
                    // if lock == false { self.wait(); } 
                    // else {
                    if lock{
                        self.update_time();
                        self.information.lock_sell += 1;
                        self.try_sell();
                        self.information.sell += 1;
                    }
                }
                self.update_time();
            }
        }
        */
    }
    
    fn find_min_price(&mut self, mode: Mode, gk: usize) -> Value{
        let mut price_market: Value = Value::new_min();
        let x = match mode {
            Mode::Buy => 0,
            Mode::Sell => 1,
        };
        for i in 0..self.prices[x].len(){ //market
            if price_market.val > self.prices[x][i][gk]{
                price_market.val = self.prices[x][i][gk];
                price_market.market = i;
            } else if price_market.val == self.prices[x][i][gk]{
                let num = rand::thread_rng().gen_range(0..100);
                if num % 2 == 0 {
                    price_market.val = self.prices[x][i][gk];
                    price_market.market = i;
                }
            }
        }
        price_market
    }
    
    fn find_max_price(&mut self, mode: Mode, gk: usize) -> Value{
        let mut price_market: Value = Value::new_max();
        let x = match mode {
            Mode::Buy => 0,
            Mode::Sell => 1,
        };
        for i in 0..self.prices[x].len(){ //market
            if price_market.val < self.prices[x][i][gk]{
                price_market.val = self.prices[x][i][gk];
                price_market.market = i;
            } else if price_market.val == self.prices[x][i][gk]{
                let num = rand::thread_rng().gen_range(0..100);
                if num % 2 == 1 {
                    price_market.val = self.prices[x][i][gk];
                    price_market.market = i;
                }
            }
        }
        price_market
    }
    
    fn find_mid_price(&mut self, mode: Mode, gk: usize) -> Value {
        let mut price_market: Value = Value::new_max();
        let min = self.find_min_price(mode.clone(), gk).market;
        let max = self.find_max_price(mode.clone(), gk).market;
        
        let mut v: Vec<usize>= vec![0,1,2];
        v.retain(|&x| x!=min && x!=max);
        
        let x = match mode {
            Mode::Buy => 0,
            Mode::Sell => 1,
        };
    
        if self.prices[x][v[0]][gk] < 0.0 { v[0] = min; }
        
        price_market.market = v[0];
        price_market.val = self.prices[x][v[0]][gk];
        price_market
    }
    
    fn chose(&mut self, count: i32, index: usize, mode: Mode) -> Value{
        if count%3==0{ self.find_min_price(mode, index) }
        else if count%3==1 { self.find_max_price(mode, index) } 
        else { self.find_mid_price(mode, index) }
    }
    
    fn try_lock_buy(&mut self, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, qty: f32) -> bool{ 
        let string: Result<String, LockBuyError>; //token
        let offer: f32;
        let min_bid_offer: Result<f32, MarketGetterError>;
            
        if qty > 0.0{
            min_bid_offer = market.borrow_mut().get_buy_price(gk, qty);
            if min_bid_offer.is_ok(){
                offer = min_bid_offer.clone().unwrap() + 0.8293 ;
                let last_lock = self.token_buy.len();
                if offer <=0.0 || offer > self.goods[0].get_qty() || (last_lock>0 && offer > self.token_buy[last_lock-1].new_qty_euro){
                    //prevent InsufficientGoodQuantity - buy + seeing that i want to do 2 lock and buy in the future
                    return false;
                }
            } else { panic!("Market error: {:?}", min_bid_offer); }

            string = market.borrow_mut().lock_buy(gk, qty, offer, self.get_name().clone());
            if let Ok(token) = string { 
                println!("want to buy: {} -> {:?}", gk, market.borrow_mut().get_name());
                let new_qty_euro = self.goods[0].get_qty() - offer; //how much EUR i have after lock (NOT change yet)
                let new_qty_gk_buy = self.goods[get_index_by_goodkind(&gk)].get_qty() + qty;
                self.token_buy.push(Locking{ token, market: market.clone(), time: -1, kind: gk, qty, new_qty_euro, new_qty_gk: new_qty_gk_buy }); 
            } else { panic!("{} -> {:?}", market.borrow_mut().get_name(), string); }
            true
        } else {
            false
        }
    }
    
    fn try_buy(&mut self) -> bool{
        //index of token_buy is always = 0 -> trader buys goods in order of locking -> once a good is buying, the relative token is removed
        //println!("{}", self.token_buy[0]);
        let mut result = false;
        if self.token_buy[0].time <10 {
            let token = self.token_buy[0].token.clone();
            let market = self.token_buy[0].market.clone();
            let gk = self.token_buy[0].kind;
            let qty = self.token_buy[0].qty;
        
            let buy = market.borrow_mut().buy(token, &mut self.goods[0]);
            if buy.is_err(){ panic!("Buy Error: {:?}", buy); }
            let res = self.goods[get_index_by_goodkind(&gk)].merge(Good::new(gk, qty));
            if res.is_err(){ panic!("Merge Error: {:?}", res); }
                        
            print!("buy {:?} with {}: {:?}\t", gk, market.borrow_mut().get_name(), buy);
            println!("{} -- {}\n", self.goods[0], self.goods[get_index_by_goodkind(&gk)]);
            self.update_all_prices(); 
            result = true;
        }
        self.token_buy.remove(0);
        result
    }
    
    fn try_lock_sell(&mut self, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, qty: f32) -> bool{
        let string: Result<String, LockSellError>; //token
        let min_offer : Result<f32, MarketGetterError>;
        let offer: f32;
        
        if qty > 0.0{
            min_offer = market.borrow_mut().get_sell_price(gk, qty);
            //let last_lock = self.token_sell.len();
            //println!("{}",last_lock);
            if min_offer.is_ok(){
                offer = min_offer.clone().unwrap() - min_offer.clone().unwrap()/10.0; 
                println!("{} AND {}", offer, self.goods[get_index_by_goodkind(&gk)].get_qty());
                if offer<=0.0 || offer > market.borrow_mut().get_goods()[0].quantity{ 
                    return false; 
                } 
                for i in self.token_sell.iter(){
                    if i.to_owned().kind == gk &&  offer > i.to_owned().new_qty_gk{
                        return false;
                    }
                }
            } else { panic!("Market error: {:?}", min_offer); }
            
            string = market.borrow_mut().lock_sell(gk, qty, offer, self.get_name().clone());
            if let Ok(token) = string { 
                println!("want to sell: {} -> {:?}; {} for {}", gk, market.borrow_mut().get_name(), qty, offer);
                let new_qty_euro = self.goods[0].get_qty() + offer;
                let new_qty_gk_sell =  self.goods[get_index_by_goodkind(&gk)].get_qty() - qty;
                self.token_sell.push(Locking{token, market: market.clone(), time: 0, kind: gk, qty, new_qty_euro, new_qty_gk: new_qty_gk_sell});
            } else { panic!("{:?}", string); }
            true
        } else {
            false
        }
    }
   
    fn try_sell(&mut self) -> bool{
        // println!("{}", self.token_sell[0]);
        let mut result = false;
        if self.token_sell[0].time <10 {
            let token = self.token_sell[0].token.clone();
            let market = self.token_sell[0].market.clone();
            let gk = self.token_sell[0].kind;
            let qty = self.token_sell[0].qty;
        
            let sell = market.borrow_mut().sell(token, &mut self.goods[get_index_by_goodkind(&gk)]);
            if sell.is_err(){ panic!("Sell Error: {:?}", sell); }
            let res = self.goods[0].merge(Good::new(GoodKind::EUR, qty));
            if res.is_err(){ panic!("Merge Error: {:?}", res); }
                        
            print!("sell {:?} with {}: {:?}\t", gk, market.borrow_mut().get_name(), sell);
            println!("{} -- {}\n", self.goods[0], self.goods[get_index_by_goodkind(&gk)]);
            self.update_all_prices(); 
            result = true;
        }
        self.token_sell.remove(0);
        result
    }
    
    fn generate_qty(&mut self, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, mode: Mode) -> f32{
        let mut max = 200.0;
        let min = 5.0;
        let mut max_reached: f32;
        let mut qty: f32;
        let x = match mode {
            Mode::Buy =>{ 0 }, //arbitrary 
            Mode::Sell => { //sell a random qty of gk that i'm sure trader posses
                max = self.goods[get_index_by_goodkind(&gk)].get_qty() - (self.goods[get_index_by_goodkind(&gk)].get_qty()*0.3);
                1
            },
        };
        if market.borrow_mut().get_name() == "RCNZ" { //MaxAllowedLocksReached for RCNZ
            max_reached = market.borrow_mut().get_goods()[get_index_by_goodkind(&gk)].quantity;
            max = max_reached - (max_reached*0.2)
        }
        if max < min { return 0.0; } 
        else { qty = rand::thread_rng().gen_range(min.. get_max(max, 200.0)); }
        if x == 0{
            let check = market.borrow_mut().get_goods();
            for x in check.iter(){ // check to prevent InsufficientGoodQuantityAvailable - lockbuy and locksell
                if x.good_kind == gk && x.quantity < qty { qty = x.quantity-(x.quantity*0.3); } 
                //qty in this way can be = 0 -> if i have the possibility in strat1 jump to sell
            }
        } else {
            let check = self.goods[get_index_by_goodkind(&gk)].get_qty();
            if qty > check {qty = check - (check*0.3)};
        }
        qty
    }

    pub fn prova(&self) -> f32 { self.goods.iter().map(|good| convert_to_eur(good)).sum() }

    fn update_time(&mut self){
        for i in 0..self.token_buy.len(){
            self.token_buy[i].time += 1;
        }
        for i in 0..self.token_sell.len(){
            self.token_sell[i].time += 1;
        } 
    }
}

fn get_max(a: f32, b: f32) -> f32{ if a > b { a } else { b } }

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

fn convert_to_eur(g: &Good) -> f32 {
    match g.get_kind() {
        GoodKind::EUR => g.get_qty(),
        GoodKind::USD => g.get_qty() / DEFAULT_EUR_USD_EXCHANGE_RATE,
        GoodKind::YEN => g.get_qty() / DEFAULT_EUR_YEN_EXCHANGE_RATE,
        GoodKind::YUAN => g.get_qty() / DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    }
}
extern crate chrono;

use std::borrow::Borrow;
use chrono::Local;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write};
use std::path::Path;
use std::rc::Rc;
use rand::Rng;
use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{GoodKind as Gk, GoodKind};
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError};
use unitn_market_2022::good::consts::{DEFAULT_EUR_YUAN_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_USD_EXCHANGE_RATE, STARTING_CAPITAL};
use unitn_market_2022::market::good_label::GoodLabel;

//external libraries
use sha256::digest;
use unitn_market_2022::good::good_error::GoodSplitError;

pub struct ZSE {
    pub name: String,
    pub goods: [Good; 4],
    pub prices_sell: [f32; 4],
    pub prices_buy: [f32; 4],
    pub lock_buy: [Lock; 4],
    pub lock_sell: [Lock; 4],
    pub locked_qty: [f32; 4],
    pub token_sell: HashMap<String, bool>,
    pub token_buy : HashMap<String,bool>,
    pub markets: Vec<dyn Notifiable>,
}

pub struct Lock {
    pub lock: [Contract; MAXLOCK],
    pub last: i32,
}

pub struct Contract {
    pub token: String,
    pub quantity: f32,
    pub price: f32,
    pub lock_counter:i32,
}

#[derive(Copy, Clone)]
pub enum Mode {
    Buy,
    Sell,
}

const MAXLOCK :usize = 3; //DO NOT TOUCH
const MAXTIME :i32 = 15; //DO NOT TOUCH
const PATH_LOG: &str = "log_ZSE.txt";

impl Notifiable for ZSE{
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.markets.push(*subscriber);
    }

    fn on_event(&mut self, event: Event) {
        //internal event despite the kind of event
        self.increment_lock_counter_and_reset();
        let kind = event.clone().kind;
        // todo take actions based on event kind
        //notify others
        match kind {
            EventKind::Wait =>{} //skip
            _ => {
                for market in self.markets.iter_mut() {
                    market.on_event(event.clone());
                }
            }
        }

    }
}

impl Market for ZSE{
    fn new_random() -> Rc<RefCell<dyn Market>> where Self: Sized {
        let mut remaining =  STARTING_CAPITAL;
        //create random float number
        let mut tmp = vec![0.0;4];
        let mut rng = rand::thread_rng();
        let mut random_float;
        for i in 0..3{
            random_float = rng.gen_range(0.0..remaining);
            tmp[i] = random_float;
            remaining -= random_float;
        }
        tmp[3] = remaining;
        let market = ZSE {
            name: "ZSE".to_string(),
            goods: [
                Good::new(Gk::EUR, tmp[0]),
                Good::new(Gk::USD, tmp[1]*DEFAULT_EUR_USD_EXCHANGE_RATE),
                Good::new(Gk::YEN, tmp[2]*DEFAULT_EUR_YEN_EXCHANGE_RATE),
                Good::new(Gk::YUAN, tmp[3]*DEFAULT_EUR_YUAN_EXCHANGE_RATE),
            ],
            prices_sell: [
                1.0,
                DEFAULT_EUR_USD_EXCHANGE_RATE,
                DEFAULT_EUR_YEN_EXCHANGE_RATE,
                DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            ],
            prices_buy: [
                1.0,
                DEFAULT_EUR_USD_EXCHANGE_RATE,
                DEFAULT_EUR_YEN_EXCHANGE_RATE,
                DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            ],
            //intialize lock_buy with all empty string
            lock_buy: [Lock::new(), Lock::new(), Lock::new(), Lock::new()],
            lock_sell: [Lock::new(), Lock::new(), Lock::new(), Lock::new()],
            locked_qty: [0.0;4],
            token_sell: HashMap::new(),
            token_buy : HashMap::new(),
            markets: Vec::new(),

        };
        //create file todo SHOULD BE A FUNCTION IDK HOW TO CALL IT INSIDE NEW
        init_file();

        //TODO format values
        let logcode = format!(
            "MARKET INITIALIZATION \n EUR: {:+e} \n USD: {:+e} \n YEN: {:+e} \n YUAN: {:+e} \n END MARKET INITIALIZATION",
            tmp[0], tmp[1], tmp[2], tmp[3]);
        print_metadata(logcode);

        Rc::new(RefCell::new(market))
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let market = ZSE {
            name: "ZSE".to_string(),
            goods: [
                Good::new(Gk::EUR, eur),
                Good::new(Gk::USD, usd),
                Good::new(Gk::YEN, yen),
                Good::new(Gk::YUAN, yuan),
            ],
            prices_sell: [
                1.0,
                DEFAULT_EUR_USD_EXCHANGE_RATE,
                DEFAULT_EUR_YEN_EXCHANGE_RATE,
                DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            ],
            prices_buy: [
                1.0,
                DEFAULT_EUR_USD_EXCHANGE_RATE,
                DEFAULT_EUR_YEN_EXCHANGE_RATE,
                DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            ],
            lock_buy: [Lock::new(), Lock::new(), Lock::new(), Lock::new()],
            lock_sell: [Lock::new(), Lock::new(), Lock::new(), Lock::new()],
            locked_qty: [0.0;4],
            token_sell: HashMap::new(),
            token_buy : HashMap::new(),
            markets: Vec::new(),
        };
        init_file();
        //Create buffer
        let logcode = format!(
            "MARKET INITIALIZATION \n EUR: {:+e} \n USD: {:+e} \n YEN: {:+e} \n YUAN: {:+e} \n END MARKET INITIALIZATION",
            eur, usd, yen, yuan);
        print_metadata(logcode);

        Rc::new(RefCell::new(market))
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let file = File::open(path);
        if file.is_err(){
            return Self::new_random();
        }
        let mut eur = 0.0;
        let mut usd = 0.0;
        let mut yen = 0.0;
        let mut yuan = 0.0;
        //create BufferedReader
        let reader = std::io::BufReader::new(file.unwrap());
        for line in reader.lines(){
            let line = line.unwrap();
            let mut split = line.split(" ");
            let good = split.next().unwrap();
            let quantity = split.next().unwrap();
            let quantity = quantity.parse::<f32>().unwrap();
            match good {
                "EUR" => eur = quantity,
                "USD" => usd = quantity,
                "YEN" => yen = quantity,
                "YUAN" => yuan = quantity,
                _ => {}
            }
        }
        //TODO print market init values into file
        init_file();
        //create buffer
        let logcode = format!(
            "MARKET INITIALIZATION \n EUR: {:+e} \n USD: {:+e} \n YEN: {:+e} \n YUAN: {:+e} \n END MARKET INITIALIZATION",
            eur, usd, yen, yuan);
        print_metadata(logcode);
        Self::new_with_quantities(eur, yen, usd, yuan)
    }

    fn get_name(&self) -> &'static str {
        "ZSE"
    }

    fn get_budget(&self) -> f32 {
        self.goods.iter().map(|good| Self::convert_to_eur(good)).sum()
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity<0.0{
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }
        let internal_quantity = self.get_quantity_by_goodkind(&kind);
        if internal_quantity < quantity{
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable { requested_good_kind: kind, requested_good_quantity: quantity, available_good_quantity: internal_quantity});
        }
        let discount = quantity/self.get_quantity_by_goodkind(&kind) * 10.0; //10% off is max discount
        let price = self.get_price_buy_by_goodkind(&kind);

        Ok(price - price*discount/100.0)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity< 0.0{
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }
        let x = self.get_price_sell_by_goodkind(&kind);

        return Ok((x+x*0.02)*quantity);
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        let mut goods = Vec::new();
        for good in self.goods.iter() {
            let label = GoodLabel  {
                good_kind: good.get_kind(),
                quantity: good.get_qty(),
                exchange_rate_buy: self.get_price_buy_by_goodkind(&good.get_kind()),
                exchange_rate_sell: self.get_price_sell_by_goodkind(&good.get_kind()),
            };
            goods.push(label);
        }
        goods
    }

    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        let logcode = format!("LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-ERROR",trader_name.clone(),kind_to_buy,quantity_to_buy,bid);
        let index = self.get_index_by_goodkind(&kind_to_buy);
        if quantity_to_buy < 0.0{
            print_metadata(logcode);
            return Err(LockBuyError::NonPositiveQuantityToBuy {negative_quantity_to_buy: quantity_to_buy});
        }
        if bid < 0.0{
            print_metadata(logcode);
            return Err(LockBuyError::NonPositiveBid {negative_bid: bid});
        }
        //3 skippata implementiamo multiple locks
        if self.lock_buy[index].last == MAXLOCK as i32{
            print_metadata(logcode);
            return Err(LockBuyError::MaxAllowedLocksReached);
        }
        if (self.goods[index].get_qty() - self.locked_qty[index] -2.0) < quantity_to_buy{
            print_metadata(logcode);
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {requested_good_kind : kind_to_buy.clone(), requested_good_quantity : quantity_to_buy, available_good_quantity : self.goods[index].get_qty()})
        }
        let minimum_bid = self.get_buy_price(kind_to_buy.clone(),quantity_to_buy);
        match minimum_bid {
            Ok(minimum) => {
                if minimum > bid {
                    print_metadata(logcode);
                    return Err(LockBuyError::BidTooLow {requested_good_kind:kind_to_buy, requested_good_quantity:quantity_to_buy, low_bid:bid , lowest_acceptable_bid: minimum});
                }
            }
            Err(e) => { panic!("Errore generazione minima offerta accettabile in acquisto") }
        }

        let token = self.hash(&kind_to_buy,quantity_to_buy,bid,&trader_name);

        //Update lock

        self.lock_buy[index].insert(&token, quantity_to_buy,bid);
        self.lock_buy[index].last += 1;
        self.locked_qty[index] += quantity_to_buy;

        //insert into Hashmap
        self.token_buy.insert(token.clone(), true);


        println!("{} {} {} {}",kind_to_buy,quantity_to_buy,bid,trader_name);
        //write into logfile
        let logcode = format!("LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-TOKEN:{}",trader_name.clone(),kind_to_buy,quantity_to_buy,bid,token.clone());
        print_metadata(logcode);
        Ok(token)
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        let logcode = format!("BUY-TOKEN:{}-ERROR",token.clone());
        if !self.token_buy.contains_key(&*token){
            print_metadata(logcode);
            return Err(BuyError::UnrecognizedToken {unrecognized_token : token});
        }
        if self.token_buy.contains_key(&*token) && !self.token_buy[&token] {
            print_metadata(logcode);
            return Err(BuyError::ExpiredToken {expired_token : token});
        }
        if cash.get_kind() != Gk::EUR{
            print_metadata(logcode);
            return Err(BuyError::GoodKindNotDefault { non_default_good_kind : cash.get_kind()});
        }
        let (gk, pos) = self.get_kind_by_token(&token, Mode::Buy);
        let index= self.get_index_by_goodkind(&gk);
        let agreed_quantity = self.lock_buy[index].lock[pos].quantity;
        if cash.get_qty() < agreed_quantity{
            print_metadata(logcode);
            return Err(BuyError::InsufficientGoodQuantity {contained_quantity : cash.get_qty() , pre_agreed_quantity: agreed_quantity})
        }

        //buy good
        let _ = match cash.split(cash.get_qty()) {
            Ok(profit) => self.goods[0].merge(profit),
            Err(e) => panic!("Errore nella split: {:?}", e),
        };
        //remove lock that was in place
        self.remove(token.clone(),index, pos, Mode::Buy);
        //notify

        //update price
        self.fluctuate();
        //return
        let mut ret = Err(GoodSplitError::NotEnoughQuantityToSplit);
        while ret.is_err() {
            ret = self.goods[index].split(agreed_quantity);
        }
        self.locked_qty[index] -= agreed_quantity;

        //write into logfile
        let logcode = format!("BUY-TOKEN:{}-OK",token.clone());
        print_metadata(logcode);
        Ok(ret.unwrap())
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        let logcode = format!("LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-ERROR",trader_name.clone(),kind_to_sell,quantity_to_sell,offer);
        let index = self.get_index_by_goodkind(&kind_to_sell);
        if quantity_to_sell < 0.0{
            print_metadata(logcode);
            return Err(LockSellError::NonPositiveQuantityToSell { negative_quantity_to_sell : quantity_to_sell});
        }
        if offer < 0.0{
            print_metadata(logcode);
            return Err(LockSellError::NonPositiveOffer { negative_offer : offer});
        }
        if self.lock_sell[index].last == MAXLOCK as i32{
            print_metadata(logcode);
            return Err(LockSellError::MaxAllowedLocksReached);
        }

        if self.goods[0].get_qty() < offer{
            print_metadata(logcode);
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, available_good_quantity: self.goods[0].get_qty()});
        }

        let acceptable_offer = self.get_sell_price(kind_to_sell.clone(), quantity_to_sell);
        match acceptable_offer {
            Ok(acceptable_offer) => {
                if acceptable_offer < offer {
                    print_metadata(logcode);
                    return Err(LockSellError::OfferTooHigh { offered_good_kind : kind_to_sell, offered_good_quantity : quantity_to_sell, high_offer : offer, highest_acceptable_offer : acceptable_offer});
                }
            }
            Err(e) => { panic!("Errore generazione massima offerta accettabile in vendita") }
        }

        let token = self.hash(&kind_to_sell,quantity_to_sell,offer,&trader_name);

        //Update lock
        self.lock_sell[index].insert(&token, quantity_to_sell, offer);
        self.lock_sell[index].last += 1;
        //Insert into Hashmap
        self.token_sell.insert(token.clone(), true);

        //write into logfile
        let logcode = format!("LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-TOKEN:{}",trader_name.clone(),kind_to_sell,quantity_to_sell,offer,token.clone());
        print_metadata(logcode);
        Ok(token)
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        let logcode = format!("BUY-TOKEN:{}-ERROR",token.clone());
        if !self.token_sell.contains_key(&*token){
            print_metadata(logcode);
            return Err(SellError::UnrecognizedToken {unrecognized_token : token});
        }
        if self.token_sell.contains_key(&*token) && !self.token_sell[&token] {
            print_metadata(logcode);
            return Err(SellError::ExpiredToken {expired_token : token});
        }

        let (gk, pos) = self.get_kind_by_token(&token, Mode::Sell);
        let index= self.get_index_by_goodkind(&gk);
        let agreed_quantity = self.lock_sell[index].lock[pos].quantity;
        let agreed_price = self.lock_sell[index].lock[pos].price;

        if good.get_kind() != gk {
            print_metadata(logcode);
            return Err(SellError::WrongGoodKind {wrong_good_kind: good.get_kind(), pre_agreed_kind: gk});
        }

        if good.get_qty() < agreed_quantity{
            print_metadata(logcode);
            return Err(SellError::InsufficientGoodQuantity {contained_quantity : good.get_qty() , pre_agreed_quantity: agreed_quantity})
        }

        let _ = match good.split(agreed_quantity) {
            Ok(profit) => self.goods[index].merge(profit),
            Err(e) => panic!("Errore nella split: {:?}", e),
        };
        //remove lock that was in place
        self.remove(token.clone(), index, pos, Mode::Sell);

        //notify

        //update price
        self.fluctuate();
        //return
        let mut ret = Err(GoodSplitError::NotEnoughQuantityToSplit);
        while ret.is_err() {
            ret = self.goods[0].split(agreed_price);
        }

        //write into logfile
        let logcode = format!("BUY-TOKEN:{}-OK",token.clone());
        print_metadata(logcode);

        Ok(ret.unwrap())
    }
}


impl ZSE{
    fn convert_to_eur(g : & Good) -> f32 {
        match g.get_kind() {
            Gk::EUR => g.get_qty(),
            Gk::USD => g.get_qty() / DEFAULT_EUR_USD_EXCHANGE_RATE,
            Gk::YEN => g.get_qty() / DEFAULT_EUR_YEN_EXCHANGE_RATE,
            Gk::YUAN => g.get_qty() / DEFAULT_EUR_YUAN_EXCHANGE_RATE,
        }
    }

    fn get_quantity_by_goodkind(&self, kind: &GoodKind) -> f32 {
        for good in self.goods.iter(){
            if good.get_kind() == *kind{
                return good.get_qty();
            }
        }
        0.0
    }

    fn get_price_sell_by_goodkind(&self, kind: &GoodKind) -> f32 {
        for i in 0..self.goods.len(){
            if self.goods[i].get_kind() == *kind{
                return self.prices_sell[i];
            }
        }
        0.0
    }

    fn get_price_buy_by_goodkind(&self, kind: &GoodKind) -> f32 {
        for i in 0..self.goods.len(){
            if self.goods[i].get_kind() == *kind{
                return self.prices_buy[i];
            }
        }
        0.0
    }

    fn get_index_by_goodkind(&self, kind: &GoodKind) -> usize {
        for i in 0..self.goods.len(){
            if self.goods[i].get_kind() == *kind{
                return i;
            }
        }
        0
    }

    fn get_kind_by_token(&self, token: &String, mode: Mode) -> (GoodKind, usize) {
        let mut var = 0;
        let mut index = 6;
        // mode : buy or sell
        for i in 0..self.lock_buy.len() {
            for j in 0..MAXLOCK {
                let _ = match mode {
                    Mode::Buy => {
                        if self.lock_buy[i].lock[j].token == *token {
                            var = i;
                            index = j;
                            break;
                        }
                    }
                    Mode::Sell => {
                        if self.lock_sell[i].lock[j].token == *token {
                            var = i;
                            index = j;
                            break;
                        }
                    }
                };
            }
        }
        return match var {
            1 => (GoodKind::USD, index),
            2 => (GoodKind::YEN, index),
            3 => (GoodKind::YUAN, index),
            _ => (GoodKind::EUR,index),
        }
    }

    fn fluctuate(&self){
        {}
    }

    fn hash(&self,v1:&GoodKind,v2:f32,v3:f32,v4:&String) -> String{
        //Hash unta
        let a = digest(self.get_index_by_goodkind(&v1).to_string());
        let b = digest(v2.to_string());
        let c = digest(v3.to_string());
        let d = digest(v4.clone());
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<u32>();
        digest(format!("{}{}{}{}{}", a, b, c, d, salt))
    }

    fn remove(&mut self, token: String,index:usize,pos:usize, mode: Mode) {
        let _ = match mode {
            Mode::Buy => {
                self.token_buy.insert(token, false);
                self.lock_buy[index].last -= 1;
                self.lock_buy[index].lock[pos].remove();
            },
            Mode::Sell => {
                self.token_sell.insert(token, false);
                self.lock_sell[index].last -= 1;
                self.lock_sell[index].lock[pos].remove();
            }
        };
    }

    fn increment_lock_counter_and_reset(&mut self){
        for i in 0..4 {
            for j in 0..MAXLOCK{
                //increment counter
                if self.lock_buy[i].lock[j].token != "".to_string() {
                    self.lock_buy[i].lock[j].lock_counter += 1;
                }
                if self.lock_sell[i].lock[j].token != "".to_string() {
                    self.lock_sell[i].lock[j].lock_counter += 1;
                }
                //If maximum is reached, remove it
                if self.lock_buy[i].lock[j].lock_counter >= MAXTIME {
                    self.token_buy.insert(self.lock_buy[i].lock[j].token.clone(),false);
                    self.lock_buy[i].lock[j].remove();
                }
                if self.lock_sell[i].lock[j].lock_counter >= MAXTIME {
                    self.token_sell.insert(self.lock_buy[i].lock[j].token.clone(),false);
                    self.lock_sell[i].lock[j].remove();
                }
            }
        }
    }

    //TODO sto per scendere dal treno, è stra unto ma questa roba prende il tempo come vogliono loro
    fn get_time(){
        let date = Local::now();
        let atm = date.format("%Y:%m:%d:%H:%M:%S:%3f");
        println!("{}",atm);

    }
}


impl Contract{
    fn new() -> Self{
        Contract{
            token: "".to_string(),
            quantity:0.0,
            price:0.0,
            lock_counter:0,
        }
    }

    fn remove(&mut self) {
        self.token = "".to_string();
        self.quantity = 0.0;
        self.price = 0.0;
        self.lock_counter = 0;
    }

}
impl Lock {
    fn new() -> Self {
        Lock {
            lock: [Contract::new(), Contract::new(), Contract::new()],
            last: 0
        }
    }

    fn insert(&mut self, token: &String, qty: f32, price:f32) {
        for i in 0..MAXLOCK {
            if self.lock[i].token == "".to_string() {
                self.lock[i].token = token.clone();
                self.lock[i].quantity = qty;
                self.lock[i].price = price;
                return;
            }
        }
    }
}

//LOGFILE STUFF
fn init_file(){
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true) //clear file
        .open(PATH_LOG);
    match file{
        Ok(file) => file,
        Err(_) => panic!("Error opening / creating file"),
    };
}

fn print_metadata(buffer:String){
    let mut file = OpenOptions::new().append(true).open(PATH_LOG); //open
    match file { //check errors
        Ok(mut file) => {
            let date = Local::now();
            let atm = date.format("%Y:%m:%d:%H:%M:%S:%3f");
            let s = format!("ZSE|{}|{}\n",atm,buffer);
            let write = file.write_all(s.as_bytes()); //write into log
            match write {
                Ok(_) => {} //whacky
                Err(_) => println!("Error writing to file"),
            }
        }
        Err(_) => panic!("Error opening file"),
    };
}


//TODO take actions on others' event(?),
// reset market(?)
// fluctuate -> modify prices over time and quantity
// get_buy price and sell price numbers
// add on event in all functions
// modify visibility of functions and clear panics


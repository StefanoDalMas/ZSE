use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use rand::{Rng, thread_rng};

use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{Market, LockBuyError, LockSellError, BuyError, SellError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};
use crate::common;

const STARTING_QUANTITY: f32 = 100000.0;
const WINDOW_SIZE: i32 = 5; // 5 * 2 = 10 (min BFB)

pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    best_prices: Vec<Vec<BestPrice>>,
    goods: Vec<Good>,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Debug)]
enum Mode {
    Buy,  //0
    Sell, //1
}

#[derive(Clone, Debug)]
struct Lock {
    token: String,
    mode: Mode,
    market: String,
    good_kind: GoodKind,
    price: f32,
    quantity: f32,
}

#[derive(Clone)]
struct BestPrice {
    price: f32,
    quantity: f32,
    market: String
}

struct Transaction {
    lock_buy: Lock,
    lock_sell: Lock,
    deadline: i32,
    priority: f32
}

impl ZSE_Trader {
    pub fn new() -> Self {
        let name = "ZSE_Trader".to_string();
        let mut markets = Vec::new();
        markets.push(RCNZ::new_random());
        markets.push(Bfb::new_random());
        markets.push(BVCMarket::new_random());
        //markets.push(RCNZ::new_with_quantities(STARTING_QUANTITY, STARTING_QUANTITY, STARTING_QUANTITY, STARTING_QUANTITY));
        //markets.push(Bfb::new_with_quantities(STARTING_QUANTITY, STARTING_QUANTITY, STARTING_QUANTITY, STARTING_QUANTITY));
        //markets.push(BVCMarket::new_with_quantities(STARTING_QUANTITY, STARTING_QUANTITY, STARTING_QUANTITY, STARTING_QUANTITY));
        subscribe_each_other!(markets[0], markets[1], markets[2]);

        let mut remaining = STARTING_QUANTITY;
        let mut tmp = vec![0.0; 4];
        let mut random_num;

        for i in 0..3 {
            random_num = rand::thread_rng().gen_range(0.0..remaining);
            tmp[i] = random_num;
            remaining -= random_num;
        }
        tmp[3] = remaining;

        let goods = vec![
            Good::new(GoodKind::EUR, tmp[0]),
            Good::new(GoodKind::USD, tmp[1]),
            Good::new(GoodKind::YEN, tmp[2]),
            Good::new(GoodKind::YUAN, tmp[3]),
        ];

        let mut best_prices = vec![vec![BestPrice{ price: 0.0, quantity: 0.0, market: "".to_string() }; 4]; 2];
        best_prices[0] = vec![BestPrice{ price: 1000000.0, quantity: 0.0, market: "".to_string() }; 4];
        best_prices[1] = vec![BestPrice{ price: -1000000.0, quantity: 0.0, market: "".to_string() }; 4];
        let transactions = Vec::new();

        Self { name, markets, best_prices, goods, transactions }
    }

    pub fn get_name(&self) -> &String { &self.name }

    pub fn get_markets(&self) -> &Vec<Rc<RefCell<dyn Market>>> { &self.markets }

    pub fn get_budget(&self) -> f32 { self.goods.iter().map(|good| convert_to_eur(good)).sum() }

    fn update_best_prices(&mut self) {
        for mode in 0..2 {
            for good in 1..4 {
                for market in 0..3 {
                    for qty in [10.0, 50.0, 100.0] {
                        let unit_price;
                        if mode == 0 {
                            let m_good = self.markets[market].borrow().get_goods()[good].quantity;
                            if m_good > qty {
                                unit_price = match self.markets[market].borrow().get_buy_price(get_goodkind_by_index(good), qty) {
                                    Ok(price) => price / qty,
                                    Err(_) => 0.0,
                                };
                            } else {
                                unit_price = 1000000.0;
                            }
                        } else {
                            let m_eur = self.markets[market].borrow().get_goods()[0].quantity;
                            let cost = match self.markets[market].borrow().get_sell_price(get_goodkind_by_index(good), qty) {
                                Ok(price) => price * qty,
                                Err(_) => 0.0,
                            };
                            if m_eur > cost {
                                unit_price = match self.markets[market].borrow().get_sell_price(get_goodkind_by_index(good), qty) {
                                    Ok(price) => price / qty,
                                    Err(_) => 0.0,
                                };
                            } else {
                                unit_price = -1000000.0;
                            }
                        }
                        if (mode == 0 && unit_price < self.best_prices[mode][good].price) || (mode == 1 && unit_price > self.best_prices[mode][good].price) {
                            self.best_prices[mode][good].price = unit_price;
                            self.best_prices[mode][good].quantity = qty;
                            self.best_prices[mode][good].market = self.markets[market].borrow().get_name().to_string();
                        }
                    }
                }
            }
        }
    }

    fn update_priorities(&mut self) {
        // HRRN priority
        for t in &mut self.transactions {
            t.priority = (t.lock_sell.price - t.lock_buy.price) / t.deadline as f32;
        }
    }

    fn update_deadlines(&mut self) {
        for t in &mut self.transactions {
            t.deadline -= 1;
        }
        self.transactions.retain(|t| t.deadline > 0);
    }

    // Buy & Sell Lock functions
    fn lock_buy(&mut self, lock: &mut Lock) -> bool {
        let res = self.markets[get_index_by_market(&*lock.market)].borrow_mut().lock_buy(lock.good_kind, lock.quantity, lock.price * lock.quantity, "ZSE".to_string());
        match res {
            Ok(str) => {
                lock.token = str;
                true
            }
            Err(err) => {
                match err {
                    LockBuyError::BidTooLow { requested_good_kind: _, requested_good_quantity: qty, low_bid: _, lowest_acceptable_bid: minimum } => {
                        lock.price = minimum / qty;
                        self.lock_buy(lock)
                    }
                    _ => { false }
                }
            }
        }
    }

    fn lock_sell(&mut self, lock: &mut Lock) -> bool {
        let res = self.markets[get_index_by_market(&*lock.market)].borrow_mut().lock_sell(lock.good_kind, lock.quantity, lock.price * lock.quantity, "ZSE".to_string());
        match res {
            Ok(str) => {
                lock.token = str;
                true
            }
            Err(err) => {
                match err {
                    LockSellError::OfferTooHigh { offered_good_kind: _, offered_good_quantity: qty, high_offer: _, highest_acceptable_offer: maximum } => {
                        lock.price = maximum / qty;
                        self.lock_sell(lock)
                    }
                    _ => { false }
                }
            }
        }
    }

    // Buy & Sell functions
    fn buy(&mut self, lock: &Lock) -> bool {
        let res = self.markets[get_index_by_market(&*lock.market)].borrow_mut().buy(lock.token.clone(), &mut self.goods[0]);
        match res {
            Ok(good) => {
                self.goods[get_index_by_goodkind(&lock.good_kind)].merge(good).expect("Merge error in buy function");
                write_metadata(&self.goods);
                true
            }
            Err(err) => {
                match err {
                    BuyError::InsufficientGoodQuantity { .. } => { println!("Not enough money to buy!"); }
                    _ => { println!("{:?}", err); }
                }
                false
            }
        }
    }

    fn sell(&mut self, lock: &Lock) -> bool {
        let res = self.markets[get_index_by_market(&*lock.market)].borrow_mut().sell(lock.token.clone(), &mut self.goods[get_index_by_goodkind(&lock.good_kind)]);
        match res {
            Ok(good) => {
                self.goods[0].merge(good).expect("Merge error in sell function");
                write_metadata(&self.goods);
                true
            }
            Err(err) => {
                match err {
                    SellError::InsufficientGoodQuantity { .. } => { println!("Not enough {} to sell!", lock.good_kind); }
                   _ => { println!("{:?}", err); }
                }
                false
            }
        }
    }

    // Locking logic
    fn lock_best_profit(&mut self) {
        let mut best_good = 0;
        let mut best_profit = 0.0;

        for good in 1..4 {
            let profit = self.best_prices[1][good].price - self.best_prices[0][good].price;
            if profit > best_profit {
                best_good = good;
                best_profit = profit;
            }
        }

        let biggest_qty = if self.best_prices[0][best_good].quantity > self.best_prices[0][best_good].quantity {
            self.best_prices[0][best_good].quantity
        } else {
            self.best_prices[1][best_good].quantity
        };

        let deadline = if get_deadline_by_market(&self.best_prices[0][best_good].market) < get_deadline_by_market(&self.best_prices[1][best_good].market) {
            get_deadline_by_market(&self.best_prices[0][best_good].market)
        } else {
            get_deadline_by_market(&self.best_prices[1][best_good].market)
        };

        let mut transaction = Transaction {
            lock_buy: Lock {
                good_kind: get_goodkind_by_index(best_good),
                market: self.best_prices[0][best_good].market.clone(),
                price: self.best_prices[0][best_good].price,
                quantity: biggest_qty,
                token: String::new(),
                mode: Mode::Buy
            },
            lock_sell: Lock {
                good_kind: get_goodkind_by_index(best_good),
                market: self.best_prices[1][best_good].market.clone(),
                price: self.best_prices[1][best_good].price,
                quantity: biggest_qty,
                token: String::new(),
                mode: Mode::Sell
            },
            deadline,
            priority: 0.0,
        };

        if self.lock_buy(&mut transaction.lock_buy) && self.lock_sell(&mut transaction.lock_sell) {
            self.transactions.push(transaction);
        }
    }

    // Dropshipping implementation
    fn dropship(&mut self) {
        let mut transaction_index = 0;

        for i in 0..self.transactions.len() {
            if self.transactions[i].priority > self.transactions[i].priority {
                transaction_index = i;
            }
        }

        let cost_buy = self.transactions[transaction_index].lock_buy.price * self.transactions[transaction_index].lock_buy.quantity;
        if self.goods[0].get_qty() >= cost_buy {
            if self.buy(&self.transactions[transaction_index].lock_buy.clone())
            && self.sell(&self.transactions[transaction_index].lock_sell.clone()) {
                self.transactions.remove(transaction_index);
            } else {
                self.transactions[transaction_index].deadline = 0;
            }
        }

    }

    pub fn trade(&mut self) {
        let mut alpha;
        let mut bankrupt= false;
        init_file();

        while !bankrupt {
            self.update_best_prices();
            println!("...................................");
            println!("Locks: {}", self.transactions.len());
            println!("Budget: {}", self.get_budget());
            alpha = self.transactions.len() as f32 / WINDOW_SIZE as f32;
            if thread_rng().gen_range(0.0..1.0) < alpha {
                self.dropship();
            } else {
                self.lock_best_profit();
            }
            self.update_priorities();
            self.update_deadlines();
            std::thread::sleep(std::time::Duration::from_millis(500));
            bankrupt = if self.get_budget() <= 0.0 { true } else { false };
        }
    }

    // Prints for debug
    pub fn print_best_prices(&self) {
        for i in 0..self.best_prices.len() {
            for j in 1..self.best_prices[i].len() {
                print!("({}, {}, {}) ", self.best_prices[i][j].price, self.best_prices[i][j].quantity, self.best_prices[i][j].market);
            }
            println!();
        }
    }

    pub fn print_goods_trader(&self){
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

fn get_market_name(n: usize) -> String{
    let name = match n {
        0 => "RCNZ".to_string(),
        1 => "BFB".to_string(),
        2 => "BVC".to_string(),
        _ => panic!("Error in print_prices"),
    };
    name
}

fn get_deadline_by_market(m: &str) -> i32 {
    match m {
        "RCNZ" => 15,
        "Baku stock exchange" => 10,
        "BFB" => 10,
        "BVC" => 12,
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

fn get_goodkind_by_index(i: usize) -> GoodKind{
    return match i {
        1 => GoodKind::USD,
        2 => GoodKind::YEN,
        3 => GoodKind::YUAN,
        _ => GoodKind::EUR,
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

fn convert_goodquantity_to_eur(g: &GoodKind, qty: f32) -> f32 {
    match g {
        GoodKind::EUR => qty,
        GoodKind::USD => qty / DEFAULT_EUR_USD_EXCHANGE_RATE,
        GoodKind::YEN => qty / DEFAULT_EUR_YEN_EXCHANGE_RATE,
        GoodKind::YUAN => qty / DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    }
}

//writing to file

fn init_file() {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(common::PATH_LOG);
    match file {
        Ok(file) => file,
        Err(_) => panic!("Error opening / creating file"),
    };
}

fn write_metadata(goods: &Vec<Good>) {
    let file = OpenOptions::new()
        .append(true)
        .open(common::PATH_LOG);
    match file {
        Ok(mut file) => {
            //generate random metadata
            let s = format!("EUR {} USD {} YEN {} YUAN {} \n",goods[0].get_qty(),goods[1].get_qty(),goods[2].get_qty(),goods[3].get_qty());
            let write = file.write_all(s.as_bytes());
            match write {
                Ok(_) => {}
                Err(_) => println!("Error writing to file"),
            }
        }
        Err(_) => println!("Error opening file"),
    };
}
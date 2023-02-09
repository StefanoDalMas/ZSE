use std::cell::RefCell;
use std::rc::Rc;
use rand::{Rng, thread_rng};
use std::fs::OpenOptions;
use std::io::Write;

use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{Market, LockBuyError, LockSellError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};

use crate::common;

const STARTING_BUDGET: f32 = 40000.0;
const WINDOW_SIZE: i32 = 5; // 5 * 2 = 10 (min BFB)

pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    best_prices: Vec<Vec<BestPrice>>,
    goods: Vec<Good>,
    transactions: Vec<Transaction>,
    days: i32,
}

struct Lock {
    token: String,
    market: String,
    price: f32,
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
    good_kind: GoodKind,
    quantity: f32,
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
        subscribe_each_other!(markets[0], markets[1], markets[2]);

        let mut remaining = STARTING_BUDGET;
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
            Good::new(GoodKind::USD, tmp[1]  * DEFAULT_EUR_USD_EXCHANGE_RATE),
            Good::new(GoodKind::YEN, tmp[2] * DEFAULT_EUR_YEN_EXCHANGE_RATE),
            Good::new(GoodKind::YUAN, tmp[3] * DEFAULT_EUR_YUAN_EXCHANGE_RATE),
        ];

        let mut best_prices = vec![vec![BestPrice{ price: 0.0, quantity: 0.0, market: "".to_string() }; 4]; 2];
        best_prices[0] = vec![BestPrice{ price: f32::MAX, quantity: 0.0, market: "".to_string() }; 4];
        best_prices[1] = vec![BestPrice{ price: f32::MIN, quantity: 0.0, market: "".to_string() }; 4];
        let transactions = Vec::new();

        Self { name, markets, best_prices, goods, transactions, days: 0 }
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
                                    Ok(price) => if price > 0.0 { price / qty } else { f32::MAX },
                                    Err(_) => f32::MAX,
                                };
                            } else {
                                unit_price = f32::MAX;
                            }
                        } else {
                            let m_eur = self.markets[market].borrow().get_goods()[0].quantity;
                            let cost = match self.markets[market].borrow().get_sell_price(get_goodkind_by_index(good), qty) {
                                Ok(price) => price * qty,
                                Err(_) => f32::MIN,
                            };
                            if m_eur > cost {
                                unit_price = match self.markets[market].borrow().get_sell_price(get_goodkind_by_index(good), qty) {
                                    Ok(price) => price / qty,
                                    Err(_) => f32::MIN,
                                };
                            } else {
                                unit_price = f32::MIN;
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
    fn lock_buy(&mut self, t: &mut Transaction) -> bool {
        if self.days >= 5 {
            for _ in 0..5 {
                wait_one_day!();
            }
            self.days = 0;
        }
        let res = self.markets[get_index_by_market(&*t.lock_buy.market)].borrow_mut().lock_buy(t.good_kind, t.quantity, t.lock_buy.price * t.quantity, "ZSE".to_string());
        match res {
            Ok(str) => {
                t.lock_buy.token = str;
                self.days = 0;
                true
            }
            Err(err) => {
                match err {
                    LockBuyError::BidTooLow { requested_good_kind: _, requested_good_quantity: qty, low_bid: offered, lowest_acceptable_bid: minimum } => {
                        t.lock_buy.price = minimum / qty;
                        if minimum - offered < 0.00001 {
                            t.lock_buy.price += 0.00001;
                        }
                        self.days += 1;
                        self.lock_buy(t)
                    }
                    _ => { false }
                }
            }
        }
    }

    fn lock_sell(&mut self, t: &mut Transaction) -> bool {
        let res = self.markets[get_index_by_market(&*t.lock_sell.market)].borrow_mut().lock_sell(t.good_kind, t.quantity, t.lock_sell.price * t.quantity, "ZSE".to_string());
        match res {
            Ok(str) => {
                t.lock_sell.token = str;
                true
            }
            Err(err) => {
                match err {
                    LockSellError::OfferTooHigh { offered_good_kind: _, offered_good_quantity: qty, high_offer: offered, highest_acceptable_offer: maximum } => {
                        t.lock_sell.price = maximum / qty;
                        if offered - maximum > 0.00001 {
                            t.lock_sell.price -= 0.00001;
                        }
                        self.lock_sell(t)
                    }
                    _ => { false }
                }
            }
        }
    }

    // Buy & Sell functions
    fn buy(&mut self, token: String, market: usize, kind: usize) -> bool {
        let res = self.markets[market].borrow_mut().buy(token, &mut self.goods[0]);
        match res {
            Ok(good) => {
                self.goods[kind].merge(good).expect("Merge error in buy function");
                write_metadata(&self.goods);
                true
            }
            Err(_) => { false }
        }
    }

    fn sell(&mut self, token: String, market: usize, kind: usize) -> bool {
        let res = self.markets[market].borrow_mut().sell(token, &mut self.goods[kind]);
        match res {
            Ok(good) => {
                self.goods[0].merge(good).expect("Merge error in sell function");
                write_metadata(&self.goods);
                true
            }
            Err(_) => { false }
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

        let deadline_buy = get_deadline_by_market(&self.best_prices[0][best_good].market);
        let deadline_sell = get_deadline_by_market(&self.best_prices[1][best_good].market);
        let deadline = if deadline_buy < deadline_sell { deadline_buy } else { deadline_sell };

        let mut transaction = Transaction {
            lock_buy: Lock {
                market: self.best_prices[0][best_good].market.clone(),
                price: self.best_prices[0][best_good].price,
                token: String::new(),
            },
            lock_sell: Lock {
                market: self.best_prices[1][best_good].market.clone(),
                price: self.best_prices[1][best_good].price,
                token: String::new(),
            },
            good_kind: get_goodkind_by_index(best_good),
            quantity: biggest_qty,
            deadline,
            priority: 0.0,
        };

        if self.lock_buy(&mut transaction) && self.lock_sell(&mut transaction) {
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

        let cost_buy = self.transactions[transaction_index].lock_buy.price * self.transactions[transaction_index].quantity;
        if self.goods[0].get_qty() >= cost_buy {
            let index_kind = get_index_by_goodkind(&self.transactions[transaction_index].good_kind);
            let market_buy = get_index_by_market(&self.transactions[transaction_index].lock_buy.market);
            let market_sell = get_index_by_market(&self.transactions[transaction_index].lock_sell.market);
            if self.buy(self.transactions[transaction_index].lock_buy.token.clone(), market_buy, index_kind)
            && self.sell(self.transactions[transaction_index].lock_sell.token.clone(), market_sell, index_kind) {
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
            //std::thread::sleep(std::time::Duration::from_millis(200));
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
            //let s = format!("EUR {} USD {} YEN {} YUAN {} \n",goods[0].get_qty(),goods[1].get_qty(),goods[2].get_qty(),goods[3].get_qty());
            let mut s = "".to_string();
            for g in goods{
                s.push_str(&format!("{} {} ", g.get_kind(), convert_to_eur(g)));
            }
            s.push('\n');
            let write = file.write_all(s.as_bytes());
            match write {
                Ok(_) => {}
                Err(_) => println!("Error writing to file"),
            }
        }
        Err(_) => println!("Error opening file"),
    };
}

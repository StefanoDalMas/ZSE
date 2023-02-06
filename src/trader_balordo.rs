use std::cell::RefCell;
use std::rc::Rc;
use rand::{Rng, thread_rng};

use rcnz_market::rcnz::RCNZ;
use bfb::bfb_market::Bfb;
use BVC::BVCMarket;

use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{Market, LockBuyError, LockSellError, BuyError, SellError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};

const STARTING_QUANTITY: f32 = 100000.0;
const WINDOW_SIZE: i32 = 10; // min BFB

pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>,
    best_prices: Vec<Vec<BestPrice>>,
    goods: Vec<Good>,
    locks: Vec<Lock>,
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
    deadline: i32,
    good_kind: GoodKind,
    price: f32,
    quantity: f32,
    priority: f32,
}

#[derive(Clone)]
struct BestPrice {
    price: f32,
    quantity: f32,
    market: String
}

impl ZSE_Trader {
    pub fn new() -> Self {
        let name = "ZSE_Trader".to_string();
        let mut markets = Vec::new();
        markets.push(RCNZ::new_random());
        markets.push(Bfb::new_random());
        markets.push(BVCMarket::new_random());
        subscribe_each_other!(markets[0], markets[1], markets[2]);
        let prices = vec![vec![vec![1.0; 4]; 3]; 2];

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
        let locks = Vec::new();

        Self { name, markets, prices, best_prices, goods, locks }
    }

    pub fn get_name(&self) -> &String { &self.name }

    pub fn get_markets(&self) -> &Vec<Rc<RefCell<dyn Market>>> { &self.markets }

    pub fn get_prices(&self) -> &Vec<Vec<Vec<f32>>> { &self.prices }

    pub fn get_budget(&self) -> f32 { self.goods.iter().map(|good| convert_to_eur(good)).sum() }

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
        self.update_best_prices();
    }

    fn update_best_prices(&mut self) {
        for mode in 0..2 {
            for good in 1..4 {
                for market in 0..3 {
                    for qty in [10.0, 50.0, 100.0] {
                        let unit_price;
                        if mode == 0 {
                            unit_price = match self.markets[market].borrow().get_buy_price(get_goodkind_by_index(good), qty) {
                                Ok(price) => price / qty,
                                Err(_) => 0.0,
                            };
                        } else {
                            unit_price = match self.markets[market].borrow().get_sell_price(get_goodkind_by_index(good), qty) {
                                Ok(price) => price / qty,
                                Err(_) => 0.0,
                            };
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
        for lock in &mut self.locks {
            let cost = lock.price * lock.quantity;
            let profit = match lock.mode {
                Mode::Buy => {
                    convert_goodquantity_to_eur(&lock.good_kind, lock.quantity) - cost
                }
                Mode::Sell => {
                    cost - convert_goodquantity_to_eur(&lock.good_kind, lock.quantity)
                }
            };
            lock.priority = profit / lock.deadline as f32;
            lock.priority = thread_rng().gen_range(0.0..100.0);
        }
    }

    fn update_deadlines(&mut self) {
        for lock in &mut self.locks {
            lock.deadline -= 1;
        }
        self.locks.retain(|lock| lock.deadline > 0);
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
                        self.lock_buy(lock)
                    }
                    _ => { false }
                }
            }
        }
    }

    // Buy & Sell functions
    fn buy(&mut self, lock: &mut Lock) -> bool {
        let res = self.markets[get_index_by_market(&*lock.market)].borrow_mut().buy(lock.token.clone(), &mut self.goods[0]);
        match res {
            Ok(good) => {
                self.goods[get_index_by_goodkind(&lock.good_kind)].merge(good).expect("Merge error in buy function");
                true
            }
            Err(err) => {
                match err {
                    BuyError::InsufficientGoodQuantity { .. } => { println!("Not enough money to buy!"); }
                    BuyError::UnrecognizedToken { .. } => { lock.deadline = 0; }
                    _ => { println!("{:?}", err); }
                }
                false
            }
        }
    }

    fn sell(&mut self, lock: &mut Lock) -> bool {
        let res = self.markets[get_index_by_market(&*lock.market)].borrow_mut().sell(lock.token.clone(), &mut self.goods[get_index_by_goodkind(&lock.good_kind)]);
        match res {
            Ok(good) => {
                self.goods[0].merge(good).expect("Merge error in sell function");
                true
            }
            Err(err) => {
                match err {
                    SellError::InsufficientGoodQuantity { .. } => { println!("Not enough {} to sell!", lock.good_kind); }
                    SellError::UnrecognizedToken { .. } => { lock.deadline = 0; }
                    _ => { println!("{:?}", err); }
                }
                false
            }
        }
    }

    // Locking logic
    fn lock_best_profit(&mut self) {
        let mut best_buy = (BestPrice { price: 1000000.0, quantity: 0.0, market: "".to_string() }, 0);
        let mut best_sell = (BestPrice { price: -1000000.0, quantity: 0.0, market: "".to_string() }, 0);

        // Find best buy and best sell
        for mode in 0..2 {
            for good in 1..4 {
                if mode == 0 && self.best_prices[mode][good].price < best_buy.0.price {
                    best_buy = (self.best_prices[mode][good].clone(), good);
                } else if mode == 1 && self.best_prices[mode][good].price > best_sell.0.price {
                    best_sell = (self.best_prices[mode][good].clone(), good);
                }
            }
        }

        // Calculate profit for best_buy and best_sell (looks if it's from a lock or best_prices of markets)
        let cost_buy = best_buy.0.price * best_buy.0.quantity;
        let cost_sell = best_sell.0.price * best_sell.0.quantity;
        let mut profit_buy = (self.best_prices[1][best_buy.1].price * self.best_prices[1][best_buy.1].quantity) - cost_buy;
        let mut profit_sell = (cost_sell - self.best_prices[0][best_buy.1].price * self.best_prices[0][best_buy.1].quantity);

        for lock in &self.locks {
            let lock_cost = lock.price * lock.quantity;
            match lock.mode {
                Mode::Sell => {
                    if lock.good_kind == get_goodkind_by_index(best_buy.1) && (lock_cost - cost_buy) > profit_buy {
                        profit_buy = lock_cost - cost_buy;
                    }
                }
                Mode::Buy => {
                    if lock.good_kind == get_goodkind_by_index(best_sell.1) && (cost_sell - lock_cost) > profit_sell {
                        profit_sell = cost_sell - lock_cost;
                    }
                }
            }
        }

        let mut new_lock = Lock {
            token: "".to_string(),
            mode: Mode::Buy,
            market: "".to_string(),
            deadline: 0,
            good_kind: GoodKind::EUR,
            price: 0.0,
            quantity: 0.0,
            priority: 0.0,
        };
        // Choose which to lock between best_buy or best_sell
        let mut best_profit = profit_buy;
        let mut best = best_buy;
        if profit_buy < profit_sell {
            best_profit = profit_sell;
            best = best_sell;
            new_lock.mode = Mode::Sell;
        }
        new_lock.market = best.0.market.clone();
        new_lock.deadline = get_deadline_by_market(&new_lock.market);
        new_lock.good_kind = get_goodkind_by_index(best.1);
        new_lock.price = best.0.price;
        new_lock.quantity = best.0.quantity;

        match new_lock.mode {
            Mode::Buy => {
                if self.lock_buy(&mut new_lock) { self.locks.push(new_lock.clone()); }
                else { wait_one_day!(); }
            }
            Mode::Sell => {
                if self.lock_sell(&mut new_lock) { self.locks.push(new_lock.clone()) }
                else { wait_one_day!(); }
            }
        }
    }

    fn lock_random(&mut self) {
        let mut new_lock = Lock {
            token: "".to_string(),
            mode: Mode::Buy,
            market: "".to_string(),
            deadline: 0,
            good_kind: GoodKind::EUR,
            price: 0.0,
            quantity: 0.0,
            priority: 0.0,
        };

        let mode = thread_rng().gen_range(0..2);
        let good = thread_rng().gen_range(1..4);

        if mode == 0 {
            new_lock.mode = Mode::Buy;
        } else {
            new_lock.mode = Mode::Sell;
        }
        new_lock.market = self.best_prices[mode][good].market.clone();
        new_lock.deadline = get_deadline_by_market(&new_lock.market);
        new_lock.good_kind = get_goodkind_by_index(good);
        new_lock.price = self.best_prices[mode][good].price;
        new_lock.quantity = self.best_prices[mode][good].quantity;

        match new_lock.mode {
            Mode::Buy => {
                if self.lock_buy(&mut new_lock) { self.locks.push(new_lock.clone()); }
                else { wait_one_day!(); }
            }
            Mode::Sell => {
                if self.lock_sell(&mut new_lock) { self.locks.push(new_lock.clone()) }
                else { wait_one_day!(); }
            }
        }
    }

    // HRRN implementation
    fn HRRN(&mut self) {
        let mut lock_index = 0;

        for i in 1..self.locks.len() {
            if self.locks[i].priority > self.locks[lock_index].priority {
                lock_index = i;
            }
        }

        match self.locks[lock_index].mode {
            Mode::Buy => {
                if self.goods[0].get_qty() >= self.locks[lock_index].quantity * self.locks[lock_index].price {
                    if self.buy(&mut self.locks[lock_index].clone()) {
                        self.locks[lock_index].deadline = 0;
                    } else {
                        println!("Error in buy function! (HRRN)");
                    }
                }
            },
            Mode::Sell => {
                let index = get_index_by_goodkind(&self.locks[lock_index].good_kind);
                if self.goods[index].get_qty() >= self.locks[lock_index].quantity * self.locks[lock_index].price {
                    if self.sell(&mut self.locks[lock_index].clone()) {
                        self.locks[lock_index].deadline = 0;
                    } else {
                        println!("Error in sell function! (HRRN)");
                    }
                }
            },
        }
    }

    pub fn trade(&mut self) {
        let mut alpha;
        let mut bankrupt= false;

        while !bankrupt {
            self.update_all_prices();
            // print lock length and budget
            println!("...................................");
            println!("Locks: {}", self.locks.len());
            println!("Budget: {}", self.get_budget());
            alpha = self.locks.len() as f32 / WINDOW_SIZE as f32;
            if rand::thread_rng().gen_range(0.0..1.0) < alpha {
                self.HRRN();
            } else {
                if thread_rng().gen_range(0.0..1.0) < 0.6 {
                    self.lock_best_profit();
                } else {
                    self.lock_random();
                }
            }
            self.update_priorities();
            self.update_deadlines();
            std::thread::sleep(std::time::Duration::from_millis(200));
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

    pub fn print_prices(&self) {
        for i in 0..self.prices.len() {
            if i == 0 {
                println!("\nBuy prices:");
            } else {
                println!("\nSell prices:");
            }
            println!("        USD         YEN         YUAN");
            for j in 0..self.prices[i].len() {
                let name = get_market_name(j);
                print!("{}:\t", name);
                for k in 1..self.prices[i][j].len() {
                    print!("{}\t", self.prices[i][j][k]);
                }
                println!();
            }
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
use std::borrow::Borrow;
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
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};

const WINDOW_SIZE: i32 = 10; // min BFB

pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>,
    best_prices: Vec<Vec<BestPrice>>,
    goods: Vec<Good>,
    locks: Vec<Lock>,
}

enum Mode {
    Buy,  //0
    Sell, //1
}

struct Lock {
    token: String,
    mode: Mode,
    market: String,
    deadline: u32,
    good_kind: GoodKind,
    price: f32,
    quantity: f32,
    priority: u32,
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

        let mut remaining = 100000.0;
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

    fn update_all_prices(&mut self) {
        for m in &self.markets {
            let index = get_index_by_market(m.borrow_mut().get_name());
            let goods = m.borrow_mut().get_goods();
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
                        if mode == 0 {
                            let unit_price = self.markets[market].borrow().get_buy_price(get_goodkind_by_index(good), qty).unwrap() / qty;
                            if unit_price < self.best_prices[mode][good].price {
                                self.best_prices[mode][good].price = unit_price;
                                self.best_prices[mode][good].quantity = qty;
                                self.best_prices[mode][good].market = self.markets[market].borrow().get_name().to_string();
                            }
                        } else {
                            let unit_price = self.markets[market].borrow().get_sell_price(get_goodkind_by_index(good), qty).unwrap() / qty;
                            if unit_price > self.best_prices[mode][good].price {
                                self.best_prices[mode][good].price = unit_price;
                                self.best_prices[mode][good].quantity = qty;
                                self.best_prices[mode][good].market = self.markets[market].borrow().get_name().to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    fn locked_quantity(&self, good_kind: GoodKind) -> f32 {
        let mut locked = 0.0;
        for lock in &self.locks {
            if lock.good_kind == good_kind {
                locked += lock.quantity;
            }
        }
        locked
    }

    // Locking logic
    fn lock_best_profit(&mut self) {
        let mut best = BestPrice{ price: 0.0, quantity: 0.0, market: "".to_string() };

        for mode in 0..2 {
            for good in 1..4 {
                if mode == 0 {
                    for lock in self.locks {
                        if lock.mode == Mode::Sell {
                            let ppu = lock.price / lock.quantity;
                            if ppu > best.price {
                                best.price = ppu;
                                best.quantity = lock.quantity;
                                best.market = lock.market;
                            }
                        }
                    }
                    let ppu = self.best_prices[1][good].price / self.best_prices[1][good].quantity;
                    if ppu < best.price {
                        best.price = ppu;
                        best.quantity = self.best_prices[1][good].quantity;
                        best.market = self.best_prices[1][good].market.clone();
                    }
                } else {
                    for lock in self.locks {
                        if lock.mode == Mode::Buy {
                            let ppu = lock.price / lock.quantity;
                            if ppu > best.price {
                                best.price = ppu;
                                best.quantity = lock.quantity;
                                best.market = lock.market;
                            }
                        }
                    }
                    let ppu = self.best_prices[0][good].price / self.best_prices[0][good].quantity;
                    if ppu > best.price {
                        best.price = ppu;
                        best.quantity = self.best_prices[0][good].quantity;
                        best.market = self.best_prices[0][good].market.clone();
                    }
                }
            }
        }

        //Fare lock effettiva
        //Mettere deadline in base al mercato
        // BFB = 10
        // RCNZ = 15
        // BVC = 12
    }

    // HRRN implementation
    fn HRRN(&mut self) {
        let mut lock = &self.locks[0];

        for i in 1..self.locks.len() {
            if self.locks[i].priority > lock.priority {
                lock = &self.locks[i];
            }
        }

        match lock.mode {
            Mode::Buy => {
                // Esegui acquisto
            },
            Mode::Sell => {
                // Esegui vendita
            },
        }
    }

    pub fn trade(&mut self) {
        use rand::Rng;

        let mut alpha = 0.0;
        let mut bankrupt: bool = false;

        while(!bankrupt) {
            self.update_all_prices();
            alpha = self.locks.len() as f32 / WINDOW_SIZE as f32;
            if rand::thread_rng().gen_range(0.0, 100.0) < alpha {
                self.HRRN();
            } else {
                self.lock_best_profit();
            }
            bankrupt = if self.get_budget() < 0.0 { true } else { false };
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
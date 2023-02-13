use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::sync::mpsc::Sender;

use bfb::bfb_market::Bfb;
use rand::Rng;
use rcnz_market::rcnz::RCNZ;
use unitn_market_2022::good::consts::{
    DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE,
};
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{LockBuyError, LockSellError, Market, MarketGetterError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use BVC::BVCMarket;
use clap::Parser;

const STARTING_CAPITAL: f32 = 40000.0;
const NUM_LOCK: i32 = 3;

unsafe impl Send for ZSE_Trader {} //mandatory in order to pass tx to the trader DONT TOUCH --needed by the compiler
pub struct ZSE_Trader {
    name: String,
    markets: Vec<Rc<RefCell<dyn Market>>>,
    prices: Vec<Vec<Vec<f32>>>,
    goods: Vec<Good>,
    token_buy: Vec<Locking>,
    token_sell: Vec<Locking>,
    information: Data,
}
#[derive(Debug, Clone)]
enum Mode {
    Buy,
    Sell,
}
pub struct Locking {
    //keep info about tokens and locks that trader does
    token: String,
    market: Rc<RefCell<dyn Market>>,
    time: i32,
    kind: GoodKind,
    qty: f32,
    new_qty_euro: f32,
    new_qty_gk: f32,
}
impl Display for Locking {
    //for degub
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} - {}", self.token, self.market.borrow_mut().get_name(),self.time)
    }
}

#[derive(Debug, Clone, Copy)]
struct Data {
    //general info
    lock_buy: i32,
    lock_sell: i32,
    buy: i32,
    sell: i32,
    wait: i32,
}
impl Data {
    fn new() -> Data {
        Data {
            lock_buy: 0,
            lock_sell: 0,
            buy: 0,
            sell: 0,
            wait: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    //finding price
    val: f32,
    market: usize,
}
impl Value {
    fn new_max() -> Self {
        Value {
            val: 0.0,
            market: 3,
        }
    }
    fn new_min() -> Self {
        Value {
            val: STARTING_CAPITAL,
            market: 3,
        }
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.val.partial_cmp(&other.val)
    }
}

impl ZSE_Trader {
    fn default() -> Self {
        let name = "ZSE_Trader".to_string();
        let markets = Vec::new();
        let prices = vec![vec![vec![0.0; 4]; 3]; 2];
        let goods = vec![];
        let token_buy = Vec::new();
        let token_sell = Vec::new();
        let information = Data::new();
        Self {
            name,
            markets,
            prices,
            goods,
            token_buy,
            token_sell,
            information,
        }
    }
    pub fn new() -> Self {
        let mut res = Self::default();
        res.markets.push(RCNZ::new_random());
        res.markets.push(Bfb::new_random());
        res.markets.push(BVCMarket::new_random());
        subscribe_each_other!(res.markets[0], res.markets[1], res.markets[2]);
        res.goods = vec![
            Good::new(GoodKind::EUR, STARTING_CAPITAL),
            Good::new(GoodKind::USD, 0.0),
            Good::new(GoodKind::YEN, 0.0),
            Good::new(GoodKind::YUAN, 0.0),
        ];
        res
    }
    pub fn new_with_quantities(data: Vec<f32>, m1: Vec<f32>, m2: Vec<f32>, m3: Vec<f32>) -> Self {
        let mut res = Self::default();
        res.markets
            .push(RCNZ::new_with_quantities(m1[0], m1[1], m1[2], m1[3]));
        res.markets
            .push(Bfb::new_with_quantities(m2[0], m2[1], m2[2], m2[3]));
        res.markets
            .push(BVCMarket::new_with_quantities(m3[0], m3[1], m3[2], m3[3]));
        subscribe_each_other!(res.markets[0], res.markets[1], res.markets[2]);

        res.goods = vec![
            Good::new(GoodKind::EUR, data[0]),
            Good::new(GoodKind::USD, data[1] * DEFAULT_EUR_USD_EXCHANGE_RATE),
            Good::new(GoodKind::YEN, data[2] * DEFAULT_EUR_YEN_EXCHANGE_RATE),
            Good::new(GoodKind::YUAN, data[3] * DEFAULT_EUR_YUAN_EXCHANGE_RATE),
        ];
        res
    }

    pub fn get_name(&self) -> &String {
        &self.name
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

    pub fn print_goods_trader(&self) {
        for g in &self.goods {
            println!("{:?} ", g);
        }
        println!();
    }

    pub fn print_data(&self) {
        println!("{:?}", self.information);
    }

    pub fn get_qty_good_trader(&mut self, i: usize) -> f32 {
        self.goods[i].get_qty()
    }

    pub fn trade(&mut self, tx: &Sender<String>) {
        let mut count = 0;
        let mut state = true;
        while state {
            state = self.strategy(count, tx);
            count += 1;
        }
        //self.print_goods_trader();
        //self.print_data();
        //println!("tot cicli: {}", count);
    }

    pub fn strategy(&mut self, x: i32, tx: &Sender<String>) -> bool {
        let mut lock: bool;
        let mut done = 0;
        if self.get_qty_good_trader(0) > 800.0 {
            //BUY
            let index_gk_buy = rand::thread_rng().gen_range(0..18) % 3 + 1;
            let gk_buy = get_goodkind_by_index(&index_gk_buy);
            let mut count_lock_buy = 0;

            let want_buy = self.chose(x, index_gk_buy, Mode::Buy);
            let mb = &self.markets[want_buy.market].clone();
            let qty_to_buy = self.generate_qty(mb, gk_buy, Mode::Buy);

            if mb.borrow_mut().get_name() == "BVC" {
                for i in self.token_buy.iter() {
                    if i.market.borrow_mut().get_name() == "BVC" {
                        count_lock_buy += 1;
                    }
                }
            }
            if count_lock_buy < 4 {
                lock = self.try_lock_buy(mb, gk_buy, qty_to_buy);
            } else { lock = false; }

            if lock {
                self.information.lock_buy += 1;
                //println!("want to buy: {} -> {}", gk_buy, mb.borrow_mut().get_name());
            } else {
                wait_one_day!(self.markets[0], self.markets[1], self.markets[2]);
                self.information.wait += 1;
                //println!("\nWAITING LOCK-BUY\n");
            }
            self.update_time();
            self.update_all_prices();

            if self.information.lock_buy % NUM_LOCK == 2 {
                while !self.token_buy.is_empty() {
                    if self.try_buy() {
                        self.information.buy += 1;
                        write_metadata(&self.goods, tx);
                    } else {
                        wait_one_day!(self.markets[0], self.markets[1], self.markets[2]);
                        self.information.wait += 1;
                        //println!("\nWAITING BUY\n");
                    }
                    self.update_time();
                    self.update_all_prices();
                }
            }
            done = 1;
        }

        if self.get_qty_good_trader(1) > 200.0
            || self.get_qty_good_trader(2) > 200.0
            || self.get_qty_good_trader(3) > 200.0
        {
            //SELL
            let index_gk_sell = rand::thread_rng().gen_range(0..18) % 3 + 1;
            let gk_sell = get_goodkind_by_index(&index_gk_sell);
            let mut count_lock_sell = 0;

            let want_sell = self.chose(x, index_gk_sell, Mode::Sell);
            let ms = &self.markets[want_sell.market].clone();
            let qty_sell = self.generate_qty(ms, gk_sell, Mode::Sell);

            if ms.borrow_mut().get_name() == "BVC" {
                for i in self.token_sell.iter() {
                    if i.to_owned().market.borrow_mut().get_name() == "BVC" {
                        count_lock_sell += 1;
                    }
                }
            }
            if count_lock_sell < 4 {
                lock = self.try_lock_sell(ms, gk_sell, qty_sell);
            } else { lock = false; }

            if lock {
                self.information.lock_sell += 1;
                //println!("want to sell: {} of {} to {}", qty_sell, gk_sell, ms.borrow_mut().get_name());
            } else {
                wait_one_day!(self.markets[0], self.markets[1], self.markets[2]);
                self.information.wait += 1;
                //println!("\nWAITING LOCK-SELL\n");
            }
            self.update_time();
            self.update_all_prices();

            if self.information.lock_sell > 0 && self.information.lock_sell % (NUM_LOCK - 1) == 0 {
                while !self.token_sell.is_empty() {
                    if self.try_sell() {
                        self.information.sell += 1;
                        write_metadata(&self.goods, tx);
                    } else {
                        wait_one_day!(self.markets[0], self.markets[1], self.markets[2]);
                        self.information.wait += 1;
                        //println!("\nWAITING SELL\n");
                    }
                    self.update_time();
                    self.update_all_prices();
                }
            }
            done = 2;
        }
        done == 1 || done == 2
    }

    fn find_min_price(&mut self, mode: Mode, gk: usize) -> Value {
        let mut price_market: Value = Value::new_min();
        let x = match mode {
            Mode::Buy => 0,
            Mode::Sell => 1,
        };
        for i in 0..self.prices[x].len() {
            if price_market.val > self.prices[x][i][gk] {
                price_market.val = self.prices[x][i][gk];
                price_market.market = i;
            } else if price_market.val == self.prices[x][i][gk] {
                let num = rand::thread_rng().gen_range(0..100);
                if num % 2 == 0 {
                    price_market.val = self.prices[x][i][gk];
                    price_market.market = i;
                }
            }
        }
        price_market
    }

    fn find_max_price(&mut self, mode: Mode, gk: usize) -> Value {
        let mut price_market: Value = Value::new_max();
        let x = match mode {
            Mode::Buy => 0,
            Mode::Sell => 1,
        };
        for i in 0..self.prices[x].len() {
            if price_market.val < self.prices[x][i][gk] {
                price_market.val = self.prices[x][i][gk];
                price_market.market = i;
            } else if price_market.val == self.prices[x][i][gk] {
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
        let mut v: Vec<usize> = vec![0, 1, 2];
        v.retain(|&x| x != min && x != max);

        let x = match mode {
            Mode::Buy => 0,
            Mode::Sell => 1,
        };

        price_market.market = v[0];
        price_market.val = self.prices[x][v[0]][gk];
        price_market
    }

    fn chose(&mut self, count: i32, index: usize, mode: Mode) -> Value {
        if count % 3 == 0 {
            self.find_min_price(mode, index)
        } else if count % 3 == 1 {
            self.find_max_price(mode, index)
        } else {
            self.find_mid_price(mode, index)
        }
    }

    fn try_lock_buy(&mut self, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, qty: f32) -> bool {
        let string: Result<String, LockBuyError>; //token
        let offer: f32;
        let min_bid_offer: Result<f32, MarketGetterError>;
        let final_val: (f32, f32);

        if qty > 0.0 {
            min_bid_offer = market.borrow_mut().get_buy_price(gk, qty);
            if min_bid_offer.is_ok() {
                offer = min_bid_offer.clone().unwrap() + 0.8293;
                if offer <= 0.0 || offer > self.goods[0].get_qty() { return false; }
                //prevent InsufficientGoodQuantity - buy + seeing that i want to do 2 lock and buy in the future

                final_val = self.check_good_qty(gk, offer);
                if final_val.0 < 0.0 { return false; }
                string = market
                    .borrow_mut()
                    .lock_buy(gk, qty, offer, self.get_name().clone());
                if let Ok(token) = string {
                    let new_qty_euro = final_val.0 - offer; //how much EUR i have after lock (NOT change yet)
                    let new_qty_gk_buy = final_val.1 + qty;
                    self.token_buy.push(Locking {
                        token,
                        market: market.clone(),
                        time: -1,
                        kind: gk,
                        qty,
                        new_qty_euro,
                        new_qty_gk: new_qty_gk_buy,
                    });
                    return true;
                }
            }
        }
        false
    }

    fn try_buy(&mut self) -> bool {
        //index of token_buy is always = 0 -> trader buys goods in order of locking -> once a good is buying, the relative token is removed
        let mut result = false;
        if self.token_buy[0].time < 10 {
            let token = self.token_buy[0].token.clone();
            let market = self.token_buy[0].market.clone();
            let gk = self.token_buy[0].kind;
            let qty = self.token_buy[0].qty;

            let buy = market.borrow_mut().buy(token, &mut self.goods[0]);
            match buy {
                Ok(_) => {
                    let _ = self.goods[get_index_by_goodkind(&gk)].merge(Good::new(gk, qty));
                    //println!("buy {} with {} -> {}\t", gk, market.borrow_mut().get_name(), qty);
                    result = true;
                },
                Err(_) => result = false,
            }
        }
        self.token_buy.remove(0);
        result
    }

    fn try_lock_sell(&mut self, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, qty: f32) -> bool {
        let string: Result<String, LockSellError>; //token
        let min_offer: Result<f32, MarketGetterError>;
        let offer: f32;
        let final_val: (f32, f32);

        if qty > 0.0 {
            min_offer = market.borrow_mut().get_sell_price(gk, qty);
            if min_offer.is_ok() {
                offer = min_offer.clone().unwrap() - (min_offer.clone().unwrap() * 0.3);
                if offer <= 0.0
                    || offer > market.borrow_mut().get_goods()[0].quantity
                {
                    return false;
                }
                for i in self.token_sell.iter() {
                    if i.kind == gk && qty > i.to_owned().new_qty_gk {
                        return false;
                    }
                }
                final_val = self.check_good_qty(gk, offer);
                if final_val.0 < 0.0 { return false; }

                string = market
                    .borrow_mut()
                    .lock_sell(gk, qty, offer, self.get_name().clone());
                if let Ok(token) = string {
                    let new_qty_euro = final_val.0 + offer;
                    let new_qty_gk_sell = final_val.1 - qty;
                    self.token_sell.push(Locking {
                        token,
                        market: market.clone(),
                        time: -1,
                        kind: gk,
                        qty,
                        new_qty_euro,
                        new_qty_gk: new_qty_gk_sell,
                    });
                    return true;
                }
            }
        }
        false
    }

    fn try_sell(&mut self) -> bool {
        let mut result = false;
        if self.token_sell[0].time < 10 {
            let token = self.token_sell[0].token.clone();
            let market = self.token_sell[0].market.clone();
            let gk = self.token_sell[0].kind;
            let qty = self.token_sell[0].qty;

            let sell = market
                .borrow_mut()
                .sell(token, &mut self.goods[get_index_by_goodkind(&gk)]);
            match sell {
                Ok(_) => {
                    let _ = self.goods[0].merge(Good::new(GoodKind::EUR, qty));
                    //println!("sell {} with {} -> {}\t", gk, market.borrow_mut().get_name(), qty);
                    result = true;
                },
                Err(_) => result = false,
            }
        }
        self.token_sell.remove(0);
        result
    }

    fn generate_qty(&mut self, market: &Rc<RefCell<dyn Market>>, gk: GoodKind, mode: Mode) -> f32 {
        let mut max = 200.0;
        let min = 5.0;
        let mut qty: f32;
        let x = match mode {
            Mode::Buy => 0, //arbitrary
            Mode::Sell => {
                //sell a random qty of gk that i'm sure trader posses
                max = self.goods[get_index_by_goodkind(&gk)].get_qty()
                    - (self.goods[get_index_by_goodkind(&gk)].get_qty() * 0.3);
                1
            }
        };
        if max < min { return 0.0; }
        else {
            qty = rand::thread_rng().gen_range(min..get_max(max, 200.0));
        }
        if x == 0 {
            let check = market.borrow_mut().get_goods();
            for x in check.iter() {
                if x.good_kind == gk && x.quantity < qty {
                    qty = x.quantity - (x.quantity * 0.3);
                }
            }
        }
        qty
    }

    fn check_good_qty(&mut self, kind: GoodKind, offer: f32) -> (f32, f32){
        let mut final_eur = self.goods[0].get_qty();
        let mut final_gk = self.goods[get_index_by_goodkind(&kind)].get_qty();
        if !self.token_buy.is_empty() {
            if !self.token_sell.is_empty() {
                if self.token_buy[self.token_buy.len() - 1].time
                    > self.token_sell[self.token_sell.len() - 1].time
                {
                    if offer > self.token_sell[self.token_sell.len() - 1].new_qty_euro {
                        return (-1.0, -1.0);
                    }
                    final_eur = self.token_sell[self.token_sell.len() - 1].new_qty_euro;
                    if kind == self.token_sell[self.token_sell.len() - 1].kind {
                        final_gk = self.goods[get_index_by_goodkind(&kind)].get_qty();
                    }
                } else {
                    if offer > self.token_buy[self.token_buy.len() - 1].new_qty_euro {
                        return (-1.0, -1.0);
                    }
                    final_eur = self.token_buy[self.token_buy.len() - 1].new_qty_euro;
                    if kind == self.token_buy[self.token_buy.len() - 1].kind {
                        final_gk = self.goods[get_index_by_goodkind(&kind)].get_qty();
                    }
                }
            } else {
                final_eur = self.token_buy[self.token_buy.len() - 1].new_qty_euro;
                final_gk = self.goods[get_index_by_goodkind(&kind)].get_qty();
            }
        } else if !self.token_sell.is_empty() {
            final_eur = self.token_sell[self.token_sell.len() - 1].new_qty_euro;
            final_gk = self.goods[get_index_by_goodkind(&kind)].get_qty();
        }
        (final_eur, final_gk)
    }

    fn update_time(&mut self) {
        for i in 0..self.token_buy.len() {
            self.token_buy[i].time += 1;
        }
        for i in 0..self.token_sell.len() {
            self.token_sell[i].time += 1;
        }
    }
}

fn get_max(a: f32, b: f32) -> f32 {
    if a > b { a }
    else { b }
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

fn get_name_market(n: usize) -> String {
    let name = match n {
        0 => "RCNZ".to_string(),
        1 => "BFB".to_string(),
        2 => "BVC".to_string(),
        _ => panic!("Error in print_prices"),
    };
    name
}

fn get_index_by_goodkind(kind: &GoodKind) -> usize {
    match *kind {
        GoodKind::EUR => 0,
        GoodKind::USD => 1,
        GoodKind::YEN => 2,
        GoodKind::YUAN => 3,
    }
}

fn get_goodkind_by_index(i: &usize) -> GoodKind {
    match *i {
        0 => GoodKind::EUR,
        1 => GoodKind::USD,
        2 => GoodKind::YEN,
        _ => GoodKind::YUAN,
    }
}

fn convert_to_eur(good: &Good) -> f32 {
    match good.get_kind() {
        GoodKind::EUR => good.get_qty(),
        GoodKind::USD => good.get_qty() / DEFAULT_EUR_USD_EXCHANGE_RATE,
        GoodKind::YEN => good.get_qty() / DEFAULT_EUR_YEN_EXCHANGE_RATE,
        GoodKind::YUAN => good.get_qty() / DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    }
}

fn write_metadata(goods: &Vec<Good>, tx: &Sender<String>) {
    let args = crate::Args::parse();
    let mut s = "2 ".to_string();
    for g in goods {
        s.push_str(&format!("{} ", convert_to_eur(g)));
    }
    s.push('\n');
    tx.send(s).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(args.delay));
}
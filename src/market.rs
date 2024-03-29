use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::consts::{DEFAULT_EUR_YUAN_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_USD_EXCHANGE_RATE, STARTING_CAPITAL};
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError};
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::event::notifiable::Notifiable;

pub struct ZSE {
    goods: [Good; 4],
    prices_sell: [f32; 4],
    prices_buy: [f32; 4],
    lock_buy: [Lock; 4],
    lock_sell: [Lock; 4],
    locked_qty: [f32; 4],
    token: HashMap<String, bool>,
    markets: Vec<Box<dyn Notifiable>>,
    external: bool,
    conversion_timer: [[i32; 4]; 4],
}

struct Lock {
    lock: [Contract; MAXLOCK],
    last: i32,
}

struct Contract {
    token: String,
    quantity: f32,
    price: f32,
    lock_counter: i32,
}

#[derive(Copy, Clone)]
enum Mode {
    Buy,
    Sell,
}

const MAXLOCK: usize = 3;
const MAXTIME: i32 = 15;
const PATH_LOG: &str = "log_ZSE.txt";

impl Notifiable for ZSE {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.markets.push(subscriber);
    }

    fn on_event(&mut self, event: Event) {
        use unitn_market_2022::event::event::EventKind;

        self.increment_lock_counter_and_reset();
        for m in &mut self.markets {
            if !self.external {
                m.on_event(event.clone());
            }
        }

        let unit_price = event.price / event.quantity;
        let index = self.get_index_by_goodkind(&event.good_kind);
        let exchange = match event.good_kind {
            GoodKind::EUR => 1.0,
            GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
            GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
            GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
        };

        // dumping strategy
        match event.kind {
            EventKind::Bought => {
                if !self.external {
                    let diff = exchange - self.prices_buy[index];
                    self.prices_buy[index] += diff * 0.8;
                    self.internal_conversion();
                }
                if self.external && unit_price < self.prices_buy[index] {
                    self.prices_buy[index] = unit_price - (unit_price * 0.015);
                }
            },
            EventKind::Sold => {
                if !self.external {
                    let diff = self.prices_sell[index] - exchange;
                    self.prices_sell[index] -= diff * 0.8;
                    self.internal_conversion();
                }
                if self.external && unit_price > self.prices_sell[index] {
                    self.prices_sell[index] = unit_price + (unit_price * 0.015);
                }
            },
            EventKind::LockedBuy => {
                if self.external && unit_price > self.prices_sell[index] {
                    self.prices_sell[index] = unit_price + (unit_price * 0.01);
                }
            },
            EventKind::LockedSell => {
                if self.external && unit_price < self.prices_buy[index] {
                    self.prices_buy[index] = unit_price - (unit_price * 0.01);
                }
            },
            _ => {}
        };
        self.external = true;
        self.decrement_conversion_timer();
    }
}

impl Market for ZSE {
    fn new_random() -> Rc<RefCell<dyn Market>> where Self: Sized {
        use rand::Rng;

        let mut remaining = STARTING_CAPITAL as i32;
        let mut tmp = vec![0.0; 4];
        let mut random_num;

        for i in 0..3 {
            random_num = rand::thread_rng().gen_range(0..remaining);
            tmp[i] = random_num as f32;
            remaining -= random_num;
        }
        tmp[3] = remaining as f32;

        let market = ZSE {
            goods: [
                Good::new(GoodKind::EUR, tmp[0]),
                Good::new(GoodKind::USD, tmp[1] * DEFAULT_EUR_USD_EXCHANGE_RATE),
                Good::new(GoodKind::YEN, tmp[2] * DEFAULT_EUR_YEN_EXCHANGE_RATE),
                Good::new(GoodKind::YUAN, tmp[3] * DEFAULT_EUR_YUAN_EXCHANGE_RATE),
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
            locked_qty: [0.0; 4],
            token: HashMap::new(),
            markets: Vec::new(),
            external: true,
            conversion_timer: [[0; 4]; 4],
        };

        init_file();
        let logcode = format!(
            "MARKET INITIALIZATION \n EUR: {:+e} \n USD: {:+e} \n YEN: {:+e} \n YUAN: {:+e} \n END MARKET INITIALIZATION",
            tmp[0], tmp[1], tmp[2], tmp[3]);
        print_metadata(logcode);

        Rc::new(RefCell::new(market))
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let market = ZSE {
            goods: [
                Good::new(GoodKind::EUR, eur),
                Good::new(GoodKind::USD, usd),
                Good::new(GoodKind::YEN, yen),
                Good::new(GoodKind::YUAN, yuan),
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
            locked_qty: [0.0; 4],
            token: HashMap::new(),
            markets: Vec::new(),
            external: true,
            conversion_timer: [[0; 4]; 4],
        };

        init_file();
        let logcode = format!(
            "MARKET INITIALIZATION \n EUR: {:+e} \n USD: {:+e} \n YEN: {:+e} \n YUAN: {:+e} \n END MARKET INITIALIZATION",
            eur, usd, yen, yuan
        );
        print_metadata(logcode);

        Rc::new(RefCell::new(market))
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>> where Self: Sized {
        use std::fs::File;
        use std::io::BufRead;

        let file = File::open(path);
        if file.is_err() {
            return Self::new_random();
        }

        let mut eur = 0.0;
        let mut usd = 0.0;
        let mut yen = 0.0;
        let mut yuan = 0.0;

        let reader = std::io::BufReader::new(file.unwrap());
        for line in reader.lines() {
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

        Self::new_with_quantities(eur, yen, usd, yuan)
    }

    fn get_name(&self) -> &'static str {
        "ZSE"
    }

    fn get_budget(&self) -> f32 {
        self.goods.iter().map(|good| Self::convert_to_eur(good)).sum()
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity < 0.0 {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let internal_quantity = self.goods[self.get_index_by_goodkind(&kind)].get_qty();
        if  internal_quantity < quantity {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable { requested_good_kind: kind, requested_good_quantity: quantity, available_good_quantity: internal_quantity });
        }

        let price = self.prices_buy[self.get_index_by_goodkind(&kind)];
        Ok(price * quantity)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity < 0.0 {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let price = self.prices_sell[self.get_index_by_goodkind(&kind)];
        Ok(price * quantity)
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        let mut goods = Vec::new();
        let mut index;

        for good in self.goods.iter() {
            index = self.get_index_by_goodkind(&good.get_kind());
            let label = GoodLabel {
                good_kind: good.get_kind(),
                quantity: good.get_qty(),
                exchange_rate_buy: self.prices_buy[index],
                exchange_rate_sell: self.prices_sell[index],
            };
            goods.push(label);
        }

        goods
    }

    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        use unitn_market_2022::event::event::EventKind;

        self.external = false;
        self.on_event(Event { kind: EventKind::LockedBuy, quantity: quantity_to_buy, price: bid, good_kind: kind_to_buy });

        let logcode = format!("LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-ERROR", trader_name.clone(), kind_to_buy, quantity_to_buy, bid);
        let index = self.get_index_by_goodkind(&kind_to_buy);
        let minimum_bid = self.get_buy_price(kind_to_buy.clone(), quantity_to_buy).unwrap();

        if quantity_to_buy < 0.0 {
            print_metadata(logcode);
            return Err(LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: quantity_to_buy });
        }
        if bid < 0.0 {
            print_metadata(logcode);
            return Err(LockBuyError::NonPositiveBid { negative_bid: bid });
        }
        if self.lock_buy[index].last == MAXLOCK as i32 {
            print_metadata(logcode);
            return Err(LockBuyError::MaxAllowedLocksReached);
        }
        if (self.goods[index].get_qty() - self.locked_qty[index]) < quantity_to_buy {
            print_metadata(logcode);
            return Err(LockBuyError::InsufficientGoodQuantityAvailable { requested_good_kind: kind_to_buy.clone(), requested_good_quantity: quantity_to_buy, available_good_quantity: (self.goods[index].get_qty() - self.locked_qty[index]) });
        }
        if minimum_bid > bid {
            print_metadata(logcode);
            return Err(LockBuyError::BidTooLow { requested_good_kind: kind_to_buy, requested_good_quantity: quantity_to_buy, low_bid: bid, lowest_acceptable_bid: minimum_bid });
        }

        let token = self.hash(&kind_to_buy, quantity_to_buy, bid, &trader_name);

        self.lock_buy[index].insert(&token, quantity_to_buy, bid);
        self.lock_buy[index].last += 1;
        self.locked_qty[index] += quantity_to_buy;

        self.token.insert(token.clone(), true);

        let logcode = format!("LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-TOKEN:{}", trader_name.clone(), kind_to_buy, quantity_to_buy, bid, token.clone());
        print_metadata(logcode);

        Ok(token)
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        use unitn_market_2022::event::event::EventKind;

        let (gk, pos) = self.get_kind_by_token(&token, Mode::Buy);
        let index = self.get_index_by_goodkind(&gk);
        let agreed_quantity = self.lock_buy[index].lock[pos].quantity;
        let agreed_price = self.lock_buy[index].lock[pos].price;
        let logcode = format!("BUY-TOKEN:{}-ERROR", token.clone());

        self.external = false;
        self.on_event(Event { kind: EventKind::Bought, quantity: agreed_quantity, price: agreed_price, good_kind: gk.clone() });

        if !self.token.contains_key(&*token) {
            print_metadata(logcode);
            return Err(BuyError::UnrecognizedToken { unrecognized_token: token });
        }
        if self.token.contains_key(&*token) && !self.token[&token] {
            print_metadata(logcode);
            return Err(BuyError::ExpiredToken { expired_token: token });
        }
        if cash.get_kind() != GoodKind::EUR {
            print_metadata(logcode);
            return Err(BuyError::GoodKindNotDefault { non_default_good_kind: cash.get_kind() });
        }
        if cash.get_qty() < agreed_price {
            print_metadata(logcode);
            return Err(BuyError::InsufficientGoodQuantity { contained_quantity: cash.get_qty(), pre_agreed_quantity: agreed_price });
        }

        let profit = cash.split(agreed_price);
        let _ = self.goods[0].merge(profit.unwrap());

        self.remove_lock(token.clone(), index, pos, Mode::Buy);

        let ret = self.goods[index].split(agreed_quantity).unwrap();
        self.locked_qty[index] -= agreed_quantity;

        let logcode = format!("BUY-TOKEN:{}-OK", token.clone());
        print_metadata(logcode);

        Ok(ret)
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        use unitn_market_2022::event::event::EventKind;

        self.external = false;
        self.on_event(Event { kind: EventKind::LockedSell, quantity: quantity_to_sell, price: offer, good_kind: kind_to_sell });

        let logcode = format!("LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-ERROR", trader_name.clone(), kind_to_sell, quantity_to_sell, offer);
        let index = self.get_index_by_goodkind(&kind_to_sell);
        let acceptable_offer = self.get_sell_price(kind_to_sell.clone(), quantity_to_sell).unwrap();

        if quantity_to_sell < 0.0 {
            print_metadata(logcode);
            return Err(LockSellError::NonPositiveQuantityToSell { negative_quantity_to_sell: quantity_to_sell });
        }
        if offer < 0.0 {
            print_metadata(logcode);
            return Err(LockSellError::NonPositiveOffer { negative_offer: offer });
        }
        if self.lock_sell[index].last == MAXLOCK as i32 {
            print_metadata(logcode);
            return Err(LockSellError::MaxAllowedLocksReached);
        }
        if (self.goods[0].get_qty() - self.locked_qty[0]) < offer {
            print_metadata(logcode);
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, available_good_quantity: self.goods[0].get_qty() });
        }
        if acceptable_offer < offer {
            print_metadata(logcode);
            return Err(LockSellError::OfferTooHigh { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, high_offer: offer, highest_acceptable_offer: acceptable_offer });
        }

        let token = self.hash(&kind_to_sell, quantity_to_sell, offer, &trader_name);

        self.lock_sell[index].insert(&token, quantity_to_sell, offer);
        self.lock_sell[index].last += 1;
        self.locked_qty[0] += offer;

        self.token.insert(token.clone(), true);

        let logcode = format!("LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-TOKEN:{}", trader_name.clone(), kind_to_sell, quantity_to_sell, offer, token.clone());
        print_metadata(logcode);

        Ok(token)
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        use unitn_market_2022::event::event::EventKind;

        let (gk, pos) = self.get_kind_by_token(&token, Mode::Sell);
        let index = self.get_index_by_goodkind(&gk);
        let agreed_quantity = self.lock_sell[index].lock[pos].quantity;
        let agreed_price = self.lock_sell[index].lock[pos].price;

        self.external = false;
        self.on_event(Event { kind: EventKind::Sold, quantity: agreed_quantity, price: agreed_price, good_kind: good.get_kind() });

        let logcode = format!("SELL-TOKEN:{}-ERROR", token.clone());

        if !self.token.contains_key(&*token) {
            print_metadata(logcode);
            return Err(SellError::UnrecognizedToken { unrecognized_token: token });
        }
        if self.token.contains_key(&*token) && !self.token[&token] {
            print_metadata(logcode);
            return Err(SellError::ExpiredToken { expired_token: token });
        }
        if good.get_kind() != gk {
            print_metadata(logcode);
            return Err(SellError::WrongGoodKind { wrong_good_kind: good.get_kind(), pre_agreed_kind: gk });
        }
        if good.get_qty() < agreed_quantity {
            print_metadata(logcode);
            return Err(SellError::InsufficientGoodQuantity { contained_quantity: good.get_qty(), pre_agreed_quantity: agreed_quantity });
        }

        let profit = good.split(agreed_quantity);
        let _ = self.goods[index].merge(profit.unwrap());

        self.remove_lock(token.clone(), index, pos, Mode::Sell);

        let ret = self.goods[0].split(agreed_price).unwrap();
        self.locked_qty[0] -= agreed_price;

        let logcode = format!("SELL-TOKEN:{}-OK", token.clone());
        print_metadata(logcode);

        Ok(ret)
    }
}


impl ZSE {
    fn convert_to_eur(g: &Good) -> f32 {
        match g.get_kind() {
            GoodKind::EUR => g.get_qty(),
            GoodKind::USD => g.get_qty() / DEFAULT_EUR_USD_EXCHANGE_RATE,
            GoodKind::YEN => g.get_qty() / DEFAULT_EUR_YEN_EXCHANGE_RATE,
            GoodKind::YUAN => g.get_qty() / DEFAULT_EUR_YUAN_EXCHANGE_RATE,
        }
    }

    fn get_index_by_goodkind(&self, kind: &GoodKind) -> usize {
        return match *kind {
            GoodKind::EUR => 0,
            GoodKind::USD => 1,
            GoodKind::YEN => 2,
            GoodKind::YUAN => 3,
        };
    }

    fn get_kind_by_token(&self, token: &String, mode: Mode) -> (GoodKind, usize) {
        let mut var = 0;
        let mut index = 0;
        let mut array = self.lock_buy.as_ref();

        match mode {
            Mode::Buy => {},
            Mode::Sell => {
                array = self.lock_sell.as_ref();
            },
        }
        for i in 0..self.goods.len() {
            for j in 0..MAXLOCK {
                if array[i].lock[j].token == *token {
                    var = i;
                    index = j;
                    break;
                }
            }
        }

        (self.goods[var].get_kind(), index)
    }

    fn hash(&self, v1: &GoodKind, v2: f32, v3: f32, v4: &String) -> String {
        use sha256::digest;
        use rand::Rng;

        let a = digest(self.get_index_by_goodkind(&v1).to_string());
        let b = digest(v2.to_string());
        let c = digest(v3.to_string());
        let d = digest(v4.clone());
        let mut rng = rand::thread_rng();
        let salt = rng.gen::<u32>();

        digest(format!("{}{}{}{}{}", a, b, c, d, salt))
    }

    fn remove_lock(&mut self, token: String, index: usize, pos: usize, mode: Mode) {
        self.token.insert(token, false);
        let _ = match mode {
            Mode::Buy => {
                self.lock_buy[index].last -= 1;
                self.lock_buy[index].lock[pos].remove();
            }
            Mode::Sell => {
                self.lock_sell[index].last -= 1;
                self.lock_sell[index].lock[pos].remove();
            }
        };
    }

    fn increment_lock_counter_and_reset(&mut self) {
        for i in 0..4 {
            for j in 0..MAXLOCK {
                if self.lock_buy[i].lock[j].token != "".to_string() {
                    self.lock_buy[i].lock[j].lock_counter += 1;
                }
                if self.lock_sell[i].lock[j].token != "".to_string() {
                    self.lock_sell[i].lock[j].lock_counter += 1;
                }
                if self.lock_buy[i].lock[j].lock_counter >= MAXTIME {
                    self.token.insert(self.lock_buy[i].lock[j].token.clone(), false);
                    self.lock_buy[i].lock[j].remove();
                }
                if self.lock_sell[i].lock[j].lock_counter >= MAXTIME {
                    self.token.insert(self.lock_sell[i].lock[j].token.clone(), false);
                    self.lock_sell[i].lock[j].remove();
                }
            }
        }
    }

    fn internal_conversion(&mut self) {
        use rand::Rng;

        let mut max_good = self.goods[0].clone();
        let mut min_good = self.goods[0].clone();
        for g in &self.goods {
            if g.get_qty() > max_good.get_qty() {
                max_good = g.clone();
            }
            if g.get_qty() < min_good.get_qty() {
                min_good = g.clone();
            }
        }
        if min_good.get_qty() < 14000.0 && max_good.get_kind() != min_good.get_kind() {
            if self.conversion_timer[self.get_index_by_goodkind(&max_good.get_kind())][self.get_index_by_goodkind(&min_good.get_kind())] == 0 {
                let conversion_rate_from = match max_good.get_kind() {
                    GoodKind::EUR => 1.0,
                    GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
                    GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
                    GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
                };
                let conversion_rate_to = match max_good.get_kind() {
                    GoodKind::EUR => 1.0,
                    GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
                    GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
                    GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
                };

                let mut value_to_convert = 0.0;
                if max_good.get_qty() > 10000.0 {
                    value_to_convert = 10000.0;
                    self.conversion_timer[self.get_index_by_goodkind(&max_good.get_kind())][self.get_index_by_goodkind(&min_good.get_kind())] = 100;
                }

                let _ = self.goods[self.get_index_by_goodkind(&max_good.get_kind())].split(value_to_convert);
                let new_good = Good::new(min_good.get_kind(), value_to_convert * conversion_rate_to / conversion_rate_from);
                let _ = self.goods[self.get_index_by_goodkind(&min_good.get_kind())].merge(new_good);
            }
        }
    }

    fn decrement_conversion_timer(&mut self) {
        for i in 0..4 {
            for j in 0..4 {
                if self.conversion_timer[i][j] > 0 {
                    self.conversion_timer[i][j] -= 1;
                }
            }
        }
    }
}


impl Contract {
    fn new() -> Self {
        Contract {
            token: "".to_string(),
            quantity: 0.0,
            price: 0.0,
            lock_counter: 0,
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
            last: 0,
        }
    }

    fn insert(&mut self, token: &String, qty: f32, price: f32) {
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

fn init_file() {
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PATH_LOG);
    match file {
        Ok(file) => file,
        Err(_) => panic!("Error opening / creating file"),
    };
}

fn print_metadata(buffer: String) {
    use std::fs::OpenOptions;
    use std::io::Write;
    use chrono::Local;

    let file = OpenOptions::new().append(true).open(PATH_LOG);
    match file {
        Ok(mut file) => {
            let date = Local::now();
            let atm = date.format("%Y:%m:%d:%H:%M:%S:%3f");
            let s = format!("ZSE|{}|{}\n", atm, buffer);
            let write = file.write_all(s.as_bytes());
            match write {
                Ok(_) => {}
                Err(_) => println!("Error writing to file"),
            }
        }
        Err(_) => panic!("Error opening file"),
    };
}

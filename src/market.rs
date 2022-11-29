use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::rc::Rc;
use rand::Rng;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{GoodKind as Gk, GoodKind};
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError};
use unitn_market_2022::good::consts::{DEFAULT_EUR_YUAN_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_USD_EXCHANGE_RATE, STARTING_CAPITAL};
use unitn_market_2022::market::good_label::GoodLabel;

//external libraries
use sha256::digest;

pub struct ZSE {
    pub name: String,
    pub goods: [Good; 4],
    pub prices_sell: [f32; 4],
    pub prices_buy: [f32; 4],
    pub lock_buy: [Lock; 4],
    pub lock_sell: [Lock; 4],
    pub locked_qty: [f32;4],
}

pub struct Lock {
    pub lock: [String; MAXLOCK],
    pub last: i32,
}

const MAXLOCK :usize = 3;

impl Notifiable for ZSE{
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        todo!()
    }

    fn on_event(&mut self, event: Event) {
        todo!()
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
        let mut market = ZSE {
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
            lock_buy: [
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
            ],
            lock_sell: [
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
            ],
            locked_qty: [0.0;4],
        };
        Rc::new(RefCell::new(market))
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let mut market = ZSE {
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
            lock_buy: [
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
            ],
            lock_sell: [
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
                Lock { lock: [String::from(""),String::from(""),String::from("")], last: 0 },
            ],
            locked_qty: [0.0;4],

        };
        Rc::new(RefCell::new(market))
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let mut file = File::open(path);
        if file.is_err(){
            return Self::new_random();
        }
        let mut eur = 0.0;
        let mut usd = 0.0;
        let mut yen = 0.0;
        let mut yuan = 0.0;
        //create BufferedReader
        let mut reader = std::io::BufReader::new(file.unwrap());
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
        Self::new_with_quantities(eur, yen, usd, yuan)
    }


    fn get_name(&self) -> &'static str {
        todo!()
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
        //Self::fluctuate(self);
        Ok(price - price*discount/100.0)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity< 0.0{
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }
        let x = self.get_price_sell_by_goodkind(&kind);
        //Self::fluctuate(self);
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
        todo!()
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        todo!()
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        if quantity_to_sell < 0.0{
            return Err(LockSellError::NonPositiveQuantityToSell { negative_quantity_to_sell : quantity_to_sell});
        }

        if offer < 0.0{
            return Err(LockSellError::NonPositiveOffer { negative_offer : offer});
        }

        // useless code todo remove
        //if self.get_lock_sell_token_by_goodkind(&kind_to_sell) != ("".to_string()) {
        //    return Err(LockSellError::DefaultGoodAlreadyLocked { token : self.get_lock_sell_token_by_goodkind(&kind_to_sell)});
        //}

        if self.lock_sell[self.get_index_by_goodkind(&kind_to_sell)].last == MAXLOCK as i32{
            return Err(LockSellError::MaxAllowedLocksReached);
        }

        if self.goods[0].get_qty() < offer{
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, available_good_quantity: self.goods[0].get_qty()});
        }

        let acceptable_offer = self.get_sell_price(kind_to_sell.clone(), quantity_to_sell);
        match acceptable_offer {
            Ok(acceptable_offer) => {
                if acceptable_offer < offer {
                    return Err(LockSellError::OfferTooHigh { offered_good_kind : kind_to_sell, offered_good_quantity : quantity_to_sell, high_offer : offer, highest_acceptable_offer : acceptable_offer});
                }
            }
            Err(e) => { panic!("Errore generazione massima offerta accettabile") }
        }

        //Hash unta
        let v1 = digest(self.get_index_by_goodkind(&kind_to_sell).to_string());
        let v2 = digest(quantity_to_sell.to_string());
        let v3 = digest(offer.to_string());
        let v4 = digest(trader_name.clone());
        let token = digest(format!("{}{}{}{}", v1, v2, v3, v4));

        //Update lock
        let index = self.get_index_by_goodkind(&kind_to_sell);
        self.lock_sell[index].insert(&token);
        self.lock_sell[index].last += 1;

        Ok(token)
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        todo!()
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

    /*
    fn get_lock_sell_token_by_goodkind(&self, kind: &GoodKind) -> String {
        for i in 0..self.goods.len(){
            if self.goods[i].get_kind() == *kind{
                return self.lock_sell[i].clone();
            }
        }
        "".to_string()
    }

    fn get_lock_buy_token_by_goodkind(&self, kind: &GoodKind) -> String {
        for i in 0..self.goods.len(){
            if self.goods[i].get_kind() == *kind{
                return self.lock_buy[i].clone();
            }
        }
        "".to_string()
    }
     */

    fn get_index_by_goodkind(&self, kind: &GoodKind) -> usize {
        for i in 0..self.goods.len(){
            if self.goods[i].get_kind() == *kind{
                return i;
            }
        }
        0
    }

    fn fluctuate(&self){
        todo!()
    }
}

impl Lock {
    fn insert(&mut self, token: &String) {
        for i in 0..MAXLOCK {
            if self.lock[i] == "".to_string() {
                self.lock[i] = token.clone();
                return;
            }
        }
    }
}

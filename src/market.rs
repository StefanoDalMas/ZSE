use std::borrow::Borrow;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::rc::{Rc, Weak};
use rand::Rng;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{GoodKind as Gk, GoodKind};
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError};
use unitn_market_2022::good::consts::{DEFAULT_EUR_YUAN_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_USD_EXCHANGE_RATE, STARTING_QUANTITY, STARTING_CAPITAL};
use unitn_market_2022::market::good_label::GoodLabel;
use crate::wrapper::Wrapper;

pub struct ZSE{
    pub name: String,
    pub goods: [Good; 4],
}




impl Notifiable for ZSE {
    fn add_subscriber(&mut self, subscriber: Weak<RefCell<dyn Notifiable>>) {
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
        todo!()
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        todo!()
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        todo!()
    }

    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        todo!()
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        todo!()
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        todo!()
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

}
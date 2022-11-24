use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::consts::STARTING_QTY;
use unitn_market_2022::good::good_kind::{GoodKind as Gk, GoodKind};
use unitn_market_2022::market::MarketError;
use rand::Rng;


//TODO LEVARLE
pub const DEFAULT_EUR_YEN_EXCHANGE_RATE: f32 = 143.615;
pub const DEFAULT_EUR_USD_EXCHANGE_RATE: f32 = 1.03576;
pub const DEFAULT_EUR_YUAN_EXCHANGE_RATE: f32 = 7.3599;

#[derive(Debug)]
pub struct Market{
    pub name: String,
    pub goods: [Good; 4],
}

impl Market{
    pub(crate) fn new() -> Self{
        //let mut remaining =  STARTING_QTY;
        let mut remaining =  1000000.0;
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
        let mut market = Market {
            name: "ZSE".to_string(),
            goods: [
                Good::new(Gk::EUR, tmp[0]),
                Good::new(Gk::USD, tmp[1]*DEFAULT_EUR_USD_EXCHANGE_RATE),
                Good::new(Gk::YEN, tmp[2]*DEFAULT_EUR_YEN_EXCHANGE_RATE),
                Good::new(Gk::YUAN, tmp[3]*DEFAULT_EUR_YUAN_EXCHANGE_RATE),
            ],
        };
        market
    }

    pub fn new_default() -> Self{
        let val = 1000000.0;
        Market {
            name: "ZSE".to_string(),
            goods: [
                Good::new(Gk::EUR, val),
                Good::new(Gk::USD, val),
                Good::new(Gk::YEN, val),
                Good::new(Gk::YUAN, val),
            ],
        }
    }

    pub fn get_market_name(&self) -> &str{
        &self.name
    }
    pub fn get_budget(&mut self) -> Good{
        self.goods[0].clone()
    }
    //TODO
    pub fn get_buy_price(&self,kind :GoodKind, quantity: f32)-> Result<f32,MarketError> {
        Err(MarketError::NotImplemented)
    }
    pub fn get_sell_price(&self,kind :GoodKind, quantity: f32)-> Result<f32,MarketError> {
        Err(MarketError::NotImplemented)
    }

    pub fn get_goods(&self) -> Vec<Good>{
        self.goods.to_vec()
    }

    pub fn lock_trader_buy_from_market(&mut self, g: GoodKind, p: f32, q: f32,d: String) -> Result<String,MarketError>{
        Err(MarketError::NotImplemented)
    }
    pub fn trader_buy_from_market(&mut self, token: String, cash: &mut Good) -> Result<Good,MarketError>{
        Err(MarketError::NotImplemented)
    }
    pub fn lock_trader_sell_to_market(&mut self, g: GoodKind, qty: f32, price: f32,d: String) -> Result<String,MarketError>{
        Err(MarketError::NotImplemented)
    }
    pub fn trader_sell_to_market(&mut self, token: String, good: &mut Good) -> Result<Good,MarketError>{
        Err(MarketError::NotImplemented)
    }


}
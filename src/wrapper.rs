use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::market::Market;

pub struct Wrapper {
    pub markets: Vec<Rc<RefCell<dyn Market>>>
}

/*
//implement formatting for Wrapper
impl std::fmt::Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrapper {{ markets: [")?;
        for market in &self.markets {
            write!(f, "{}, ", market.borrow().borrow())?;
        }
        write!(f, "] }}")
    }
}*/


impl Wrapper{

    pub fn new() -> Self{
        Self{
            markets: Vec::new()
        }
    }

}
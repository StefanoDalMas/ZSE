#[cfg(test)]
mod test{
    use std::cell::RefCell;
    use std::rc::Rc;
    use unitn_market_2022::market::Market;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind as Gk;
    use unitn_market_2022::market::LockSellError as LSE;

    /// modify accordingly to your implementation
    use crate::market::ZSE;
    pub fn init() -> Rc<RefCell<dyn Market>>{
        ZSE::new_random()
    }
    /// modify with the max locks your implementation allows
    pub fn init_lock() -> i32 {
        3
    }

    //tests
    #[test]
    fn nonPositiveQuantityToSell(){
        let mut market = init();

        let token = market.borrow_mut().lock_sell(Gk::EUR, -100.0, 1.0, "test".to_string());
        assert_eq!(token, Err(LSE::NonPositiveQuantityToSell{negative_quantity_to_sell: -100.0}));
    }

    #[test]
    fn nonPositiveOffer() {
        let mut market = init();

        let token = market.borrow_mut().lock_sell(Gk::EUR, 100.0, -1.0, "test".to_string());
        assert_eq!(token, Err(LSE::NonPositiveOffer{negative_offer: -1.0}));
    }


    #[test]
    fn defaultGoodAlreadyLocked() {
        let mut market = init();

        let token1 = market.borrow_mut().lock_sell(Gk::EUR, 100.0, 220.0, "test".to_string());
        match token1 {
            Ok(token) => {
                let token2 = market.borrow_mut().lock_sell(Gk::EUR, 100.0, 10.0, "test".to_string());
                assert_eq!(token2, Err(LSE::DefaultGoodAlreadyLocked{ token }));
            },
            Err(e) =>{
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn maxAllowedLocksReached(){
        let mut market = init();
        let max_locks = init_lock();
        let mut token;

        for _ in 0..max_locks {
            token = market.borrow_mut().lock_sell(Gk::EUR, 100.0, 1.0, "test".to_string());
            match token {
                Ok(_) => {},
                Err(e) => panic!("Error: {:?}", e)
            }
        }

        token = market.borrow_mut().lock_sell(Gk::EUR, 100.0, 1.0, "test".to_string());
        assert_eq!(token, Err(LSE::MaxAllowedLocksReached));
    }

    #[test]
    fn insufficientDefaultGoodQuantityAvailable(){
        let mut market = init();
        let goods = market.borrow_mut().get_goods();
        let good = Good::new(Gk::USD, 1000.0);
        let available = goods[0].quantity;

        let token = market.borrow_mut().lock_sell(good.get_kind(), good.get_qty(),1000000.0, "test".to_string());
        assert_eq!(token, Err(LSE::InsufficientDefaultGoodQuantityAvailable {offered_good_kind:good.get_kind(), offered_good_quantity:good.get_qty(), available_good_quantity: available}));
    }

    #[test]
    fn offerTooHigh(){
        let mut market = init();
        let goods = market.borrow_mut().get_goods();
        let good = Good::new(Gk::USD, 1000.0);
        let available = goods[0].quantity;
        let highest_acceptable = market.borrow().get_sell_price(good.get_kind(), good.get_qty()).unwrap();

        let token = market.borrow_mut().lock_sell(good.get_kind(), good.get_qty(),available - 1.0, "test".to_string());
        assert_eq!(token, Err(LSE::OfferTooHigh {offered_good_kind:good.get_kind(), offered_good_quantity:good.get_qty(), high_offer: available - 1.0, highest_acceptable_offer: highest_acceptable}));
    }

    #[test]
    fn test_working_function(){
        let mut market = init();
        let token = market.borrow_mut().lock_sell(Gk::USD, 100.0, 1.0, "test".to_string());
        println!("Token: {:?}", token);
    }
}

#[cfg(test)]
mod test{
    use unitn_market_2022::market::Market;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind as Gk;
    use unitn_market_2022::market;
    use unitn_market_2022::market::LockSellError as LSE;
    use crate::market::ZSE;

    #[test]
    fn nonPositiveQuantityToSell(){
        let mut market = ZSE::new_random();
        //let token = market.lock_sell(super::GoodKind::EUR, 100.0, 1.0, "test".to_string()).unwrap();
        //let good = market.sell(token, &mut good).unwrap();
        //assert_eq!(good.get_quantity(), 0.0);
        //test non positive quantity
        let token = market.borrow_mut().lock_sell(Gk::EUR, -100.0, 1.0, "test".to_string());
        assert_eq!(token, Err(LSE::NonPositiveQuantityToSell{negative_quantity_to_sell: -100.0}));
    }

    #[test]
    fn nonPositiveOffer() {
        let mut market = ZSE::new_random();

        let token = market.borrow_mut().lock_sell(Gk::EUR, 100.0, -1.0, "test".to_string());
        assert_eq!(token, Err(LSE::NonPositiveOffer{negative_offer: -1.0}));
    }


    #[test]
    fn defaultGoodAlreadyLocked() {
        let mut market = ZSE::new_random();

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
        let mut market = ZSE::new_random();
        let max_locks = 3;  //modify with the max number of locks allowed
        let mut token;

        for i in 0..max_locks {
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
        let mut market = ZSE::new_random();
        let goods = market.borrow_mut().get_goods();
        let good = Good::new(Gk::USD, 1000.0);


        let token = market.borrow_mut().lock_sell(good.get_kind(), good.get_qty(),1000000.0, "test".to_string());
        //assert_eq!(token, LSE::InsufficientDefaultGoodQuantityAvailable {offered_good_kind:good.get_kind(),offered_good_quantity:good.get_qty(),available_good_quantity:market.borrow_mut().get_quantity_by_goodkind(good.get_kind())});
    }

    #[test]
    fn offerTooHigh(){

    }
}
use unitn_market_2022::market::Market;

mod market;
mod wrapper;
mod rimuovere;

fn main() {
    println!("Henlo");
    let mut market = market::ZSE::new_random();
}
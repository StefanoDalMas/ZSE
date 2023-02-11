use std::fs::File;
use std::io::{Read, Write};
use std::string::ToString;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{mem, thread};

use bfb::bfb_market::Bfb;
use clap::Parser;
use eframe::egui::plot::PlotPoint;
use eframe::{egui, run_native};
use rand::{thread_rng, Rng};
use rcnz_market::rcnz::RCNZ;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::Market;
use BVC::BVCMarket;

mod coolvisualizer;
mod trader;
mod trader_balordo;

const TX_DELAY_MS: u64 = 200;
const STARTING_BUDGET: f32 = 40000.0;

//Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = STARTING_BUDGET)]
    budget: f32,

    #[arg(short, long, default_value_t = TX_DELAY_MS)]
    delay: u64,

    #[arg(short, long, default_value = "from ZSE")]
    name: String,
}

fn main() {
    let args = Args::parse();
    //Market init
    let bfb = Bfb::new_random();
    let rcnz = RCNZ::new_random();
    let bvc = BVCMarket::new_random();
    let values_bfb = bfb.borrow().get_goods();
    let values_rcnz = rcnz.borrow().get_goods();
    let values_bvc = bvc.borrow().get_goods();

    //trader init
    let mut remaining = args.budget;
    let mut tmp = vec![0.0; 4];
    for i in 0..3 {
        tmp[i] = thread_rng().gen_range(0.0..remaining);
        remaining -= tmp[i];
    }
    tmp[3] = remaining;

    let mut trader1 = trader_balordo::ZSE_Trader::new_with_quantities(
        tmp.clone(),
        parse(&values_rcnz),
        parse(&values_bfb),
        parse(&values_bvc),
    );
    let mut trader2 = trader::ZSE_Trader::new_with_quantities(
        tmp.clone(),
        parse(&values_rcnz),
        parse(&values_bfb),
        parse(&values_bvc),
    );

    //visualizer init
    let mut visualizer = coolvisualizer::Visualizer::new();
    let mut dataset_dropship = visualizer.dataset_dropship.clone();
    let mut dataset_3m = visualizer.dataset_3m.clone();
    let mut native_options = set_native_options();

    //FIFO init
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    thread::spawn(move || {
        trader2.trade(&tx2);
    });

    thread::spawn(move || {
        trader1.trade(&tx);
    });

    thread::spawn(move || {
        let mut count = 0;
        for str in rx {
            //append data to the vector to make it visible in the plot
            let mut data = str.split_whitespace().collect::<Vec<&str>>();
            let id = data.remove(0);
            if id == "1" {
                dataset_dropship
                    .lock()
                    .unwrap()
                    .append_points(data.clone(), count as f64);
            }
            if id == "2" {
                dataset_3m
                    .lock()
                    .unwrap()
                    .append_points(data.clone(), count as f64);
            }

            //print_vector(&dataset.lock().unwrap().get_points());
            count += 1;
            thread::sleep(Duration::from_millis(args.delay));
        }
    });
    run_native(
        "Trader ZSE",
        native_options,
        Box::new(|_| Box::new(visualizer)),
    )
    .expect("Failed to run app");
}

fn set_native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(1300.0, 650.0)),
        resizable: false,
        follow_system_theme: true,
        run_and_return: false,
        ..Default::default()
    }
}

fn parse(v: &Vec<GoodLabel>) -> Vec<f32> {
    let mut res = vec![0.0; 4];
    for i in 0..4 {
        res[i] = v[i].quantity;
    }
    res
}

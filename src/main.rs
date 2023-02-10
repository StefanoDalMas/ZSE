//MAIN

use std::fs::File;
use std::sync::{Arc, mpsc, Mutex};
use std::{mem, thread};
use std::io::{Read, Write};
use std::time::Duration;
use eframe::egui::plot::PlotPoint;
use eframe::run_native;

mod coolvisualizer;
mod trader;
mod trader_balordo;

fn main() {
    let mut trader = trader_balordo::ZSE_Trader::new();
    let mut visualizer = coolvisualizer::Visualizer::new();
    let mut dataset = visualizer.dataset.clone();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        trader.trade(&tx);
    });

    thread::spawn(move || {
        let mut count = 0;
        for str in rx {
            //append data to the vector to make it visible in the plot
            dataset.lock().unwrap().append_point(PlotPoint { x: count as f64, y: coolvisualizer::get_budget(str.clone()) });
            //print_vector(&dataset.lock().unwrap().get_points());
            count += 1;
            thread::sleep(Duration::from_millis(15));
        }
    });

    run_native(
        "Trader ZSE",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(visualizer)),
    )

    /*
    for received in rx {
        println!("Got: {}", received);
    }
     */
}

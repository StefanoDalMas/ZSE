//MAIN

use std::fs::File;
use std::sync::{Arc, mpsc, Mutex};
use std::{mem, thread};
use std::io::{Read, Write};
use std::time::Duration;
use eframe::egui::plot::PlotPoint;
use eframe::{egui, run_native};

mod coolvisualizer;
mod trader;
mod trader_balordo;
const TX_DELAY_MS:u64 = 50;


fn main() {
    let mut trader1 = trader_balordo::ZSE_Trader::new();
    let mut visualizer = coolvisualizer::Visualizer::new();
    let mut dataset = visualizer.dataset.clone();
    let mut native_options = set_native_options();
    let (tx, rx) = mpsc::channel();

    // let mut trader2 = trader::ZSE_Trader::new();
    // thread::spawn(move || {
    //     trader2.trade(&tx);
    // });

    thread::spawn(move || {
        trader1.trade(&tx);
    });

    thread::spawn(move || {
        let mut count = 0;
        for str in rx {
            //append data to the vector to make it visible in the plot
            dataset.lock().unwrap().append_points(str,count as f64);
            //print_vector(&dataset.lock().unwrap().get_points());
            count += 1;
            thread::sleep(Duration::from_millis(TX_DELAY_MS));
        }
    });
    run_native(
        "Trader ZSE",
        native_options,
        Box::new(|_| Box::new(visualizer)),
    ).expect("Failed to run app");
}

fn set_native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(1300.0, 650.0)),
        resizable: false,
        follow_system_theme: true,
        run_and_return: false,
        centered: true,
        ..Default::default()
    }
}

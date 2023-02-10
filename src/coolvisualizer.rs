use eframe::{App, CreationContext, egui, Frame, run_native};
use eframe::egui::{CentralPanel, Context};
use eframe::egui::plot::{Line, Plot, PlotPoints, PlotPoint, PlotPoints::Owned};

use rand::Rng;
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use std::fmt::Formatter;
use std::thread::sleep;
use std::collections::VecDeque;
use std::fs::{OpenOptions, write};
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::mpsc::Receiver;

const WINSIZE: f32 = 100000.0;
const MIN_Y: f32 = 40000.0;
const MAX_Y: f32 = 50000.0;
const DEFAULT_DELAY_MS: u8 = 15;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = WINSIZE)]
    pub window_size: f32,

    #[arg(short, long, default_value_t = MIN_Y)]
    pub lower_bound: f32,

    #[arg(short,long, default_value_t = MAX_Y)]
    pub upper_bound: f32,

    #[arg(short, long, default_value_t = DEFAULT_DELAY_MS)]
    pub delay_ms: u8,
}

#[derive(Debug, Clone)]
pub struct Dataset {
    values :VecDeque<PlotPoint>
}

impl Dataset{
    fn new() -> Self { Dataset{ values: VecDeque::new() } }

    pub fn append_point(&mut self, value: PlotPoint) {
        self.values.retain(|x| x.x >= value.x - WINSIZE as f64);
        self.values.push_back(value);
    }

    pub fn get_points(&self) -> PlotPoints { PlotPoints::Owned(Vec::from_iter(self.values.iter().cloned())) }

    //fn consume_data(&mut self) -> Option<String> {}
}

pub struct Visualizer {
    pub dataset: Arc<Mutex<Dataset>>,
    window_y: Vec<f32>,
}


impl Visualizer {
    pub fn new() -> Self {
        let args = Args::parse();
        Visualizer {
            dataset: Arc::new(Mutex::new(Dataset::new())),
            window_y: vec![args.lower_bound,args.upper_bound],
        }
    }
}

impl App for Visualizer {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let mut plot = Plot::new("cooltrader");

            for &y in self.window_y.iter(){
                plot = plot.include_y(y);
            }

            plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(self.dataset.lock().unwrap().get_points()));
            });
        });
        ctx.request_repaint();
    }
}


pub fn get_budget(s: String) -> f64{
    let mut res = s.split_whitespace();
    let mut sum = 0.0;
    for _ in 0..4{
        res.next();
        sum += res.next().unwrap().parse::<f64>().unwrap();
    }
    sum
}

//debug functions
fn print_point(point: &PlotPoint){
    println!("x: {}, y: {}", point.x, point.y);
}

fn print_vector(vector: &PlotPoints){
    vector.points().iter().for_each(|x| print_point(x));
}
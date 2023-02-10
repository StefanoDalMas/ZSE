use eframe::{App, CreationContext, egui, Frame, run_native};
use eframe::egui::{CentralPanel, Context, SidePanel};
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

const WINSIZE: f32 = 5000.0;
const MIN_Y: f32 = 40000.0;
const MAX_Y: f32 = 80000.0;
const DEFAULT_DELAY_MS: u8 = 100;

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
    capital :VecDeque<PlotPoint>,
    eur: VecDeque<PlotPoint>,
    usd: VecDeque<PlotPoint>,
    yen: VecDeque<PlotPoint>,
    yuan: VecDeque<PlotPoint>,
}

impl Dataset{
    fn new() -> Self { Dataset{ capital: VecDeque::new(), eur: VecDeque::new(), usd: VecDeque::new(), yen: VecDeque::new(), yuan: VecDeque::new() } }

    pub fn append_points(&mut self, message :String, count:f64) {
        //remove all old data
        println!("{}",message);
        println!("REMOVING data");
        self.capital.retain(|x| x.x >= count - WINSIZE as f64 - 100.0);
        self.eur.retain(|x| x.x >= count - WINSIZE as f64 - 100.0);
        self.usd.retain(|x| x.x >= count - WINSIZE as f64 - 100.0);
        self.yen.retain(|x| x.x >= count - WINSIZE as f64 - 100.0);
        self.yuan.retain(|x| x.x >= count - WINSIZE as f64 - 100.0);
        //update all data
        println!("Removed data");
        let mut split = message.split_whitespace();
        let eur = split.clone().nth(1).unwrap().parse::<f64>().unwrap();
        let usd = split.clone().nth(3).unwrap().parse::<f64>().unwrap();
        let yen = split.clone().nth(5).unwrap().parse::<f64>().unwrap();
        let yuan = split.clone().nth(7).unwrap().parse::<f64>().unwrap();
        println!("Values are");
        println!("{} {} {} {}",eur, usd, yen, yuan);
        self.capital.push_back(PlotPoint{
            x:   count,
            y:   eur+usd+yen+yuan,
        });
        self.eur.push_back(PlotPoint{
            x:   count,
            y:   eur,
        });
        self.usd.push_back(PlotPoint{
            x:   count,
            y:   usd,
        });
        self.yen.push_back(PlotPoint{
            x:   count,
            y:   yen,
        });
        self.yuan.push_back(PlotPoint{
            x:   count,
            y:   yuan,
        });
    }

    pub fn get_points(&self) -> PlotPoints { Owned(Vec::from_iter(self.capital.iter().cloned())) }

    //fn consume_data(&mut self) -> Option<String> {}
}

pub struct Visualizer {
    pub dataset: Arc<Mutex<Dataset>>,
    window_y: Vec<f32>,
    state: String,
}


impl Visualizer {
    pub fn new() -> Self {
        let args = Args::parse();
        Visualizer {
            dataset: Arc::new(Mutex::new(Dataset::new())),
            window_y: vec![args.lower_bound,args.upper_bound],
            state: "CAPITAL".to_string(),
        }
    }
}

impl App for Visualizer {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut Frame) {
        SidePanel::left("Prova").show(ctx, |ui|{
           ui.label("Cazzo");
        });
        SidePanel::right("Prova")
            .show_separator_line(false)
            .exact_width(1000.0)
            .show(ctx, |ui|{
            let mut plot = Plot::new("cooltrader");

            for &y in self.window_y.iter(){
                plot = plot.include_y(y);
            }
            //match enum -> decide which vector to show
            plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(self.dataset.lock().unwrap().get_points()));
            });
        });
        ctx.request_repaint();
        //button handler -> enum set
    }
}


//debug functions
fn print_point(point: &PlotPoint){
    println!("x: {}, y: {}", point.x, point.y);
}

fn print_vector(vector: &PlotPoints){
    vector.points().iter().for_each(|x| print_point(x));
}
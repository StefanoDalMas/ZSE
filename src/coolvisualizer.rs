use crate::filereader;

//eframe related imports
use eframe::{App, CreationContext, egui, Frame, run_native};
use eframe::egui::{CentralPanel, Context};
use eframe::egui::plot::{Line, Plot, PlotPoints, PlotPoint};

//other imports
use rand::Rng;
use clap::Parser;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use std::fmt::Formatter;
use std::thread::sleep;


pub type data = egui::plot::PlotPoint; //might modify later idk
pub const WINSIZE:f64 = 100.0;
pub const TEST_DATASET_SIZE:i32=15000;
const MIN_Y:f64 = 0.0;
const MAX_Y:f64 = 50000.0;
const DEFAULT_DELAY_MS:u64 = 15;

struct Cooltrader{
    name:String,
    dataset:Arc<Mutex<filereader::Dataset>>,
    minmaxwindow_y:Vec<f64>,
}

impl Cooltrader{
    fn new(window_size:f64,minmaxwindow_y: Vec<f64>) -> Self{
        Cooltrader{
            name:"Cooltrader".to_string(),
            dataset:Arc::new(Mutex::new(filereader::Dataset::new(window_size))),
            minmaxwindow_y,

        }
    }
}
impl App for Cooltrader{
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| { //the window itself
            let mut plot = Plot::new("cooltrader");

            //set min and max values to force the plot to show them
            for &y in self.minmaxwindow_y.iter(){
                plot = plot.include_y(y);
            }

            plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(self.dataset.lock().unwrap().get_as_plotpoints()));
            });
        });
        ctx.request_repaint(); //in order to refresh constantly (does not put much stress into cpu)
    }
}

fn fake_data_generator() -> Vec<PlotPoint>{
    let mut seed = rand::thread_rng();
    let mut res = Vec::new();
    let size = 10;
    let mut lastx =0.0;
    let mut lasty =0.0;
    for _ in 0..size{
        let x = seed.gen_range(lastx..20.0);
        let y = seed.gen_range(lasty..20.0);
        let point = PlotPoint{x,y};
        res.push(point);
        lastx = x;
        lasty = y;
    }
    res
}

fn get_budget(s: String) -> f64{
    let mut res = s.split_whitespace();
    let mut sum = 0.0;
    for _ in 0..4{
        res.next();
        sum += res.next().unwrap().parse::<f64>().unwrap();
    }
    sum
}


//i want to add parsing from CLI to set windowsize and minmaxwindow_y, so im gonna use Clap
//default_value to have something like a builder pattern
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = WINSIZE)]
    pub window_size: f64,

    #[arg(short, long, default_value_t = MIN_Y)]
    pub min_y: f64,

    #[arg(short, long, default_value_t = MAX_Y)]
    pub y_max: f64,

    #[arg(short, long, default_value_t = DEFAULT_DELAY_MS)]
    pub delay_ms : u64,
}

//this is the main in local
pub fn try_viz(){
    let args = Args::parse();
    let mut window_y = vec![args.min_y,args.y_max];
    let mut cooltrader = Cooltrader::new(args.window_size,window_y);
    let native_options = eframe::NativeOptions::default();
    let mut thread_cooltrader = cooltrader.dataset.clone();
    cooltrader.dataset.lock().unwrap().publish_test_times(TEST_DATASET_SIZE);
    //cooltrader.dataset.lock().unwrap().append_vector(fake_data_generator()); to add some dumb data
    //eframe cannot both visualize and collect data at the same time, so we have to use a thread
    thread::spawn(move ||{
        println!("Thread started");
        let mut count =0.0;
        let mut budget = 0.0;
        let mut s;
        while count<TEST_DATASET_SIZE as f64 {
            s = thread_cooltrader.lock().unwrap().consume_one_line(); //read one line from the dataset
            match s{
                Some(s) => {
                    budget = get_budget(s);
                    //append data to the vector to make it visible in the plot
                    thread_cooltrader.lock().unwrap().append_single_plotpoint(PlotPoint{x:count,y:budget});
                    print_vector(&thread_cooltrader.lock().unwrap().get_as_plotpoints());


                    sleep(std::time::Duration::from_millis(args.delay_ms)); //delay to debug
                },
                None => println!("No more data to consume"),
            }
            count+=1.0;
        }
    });

    run_native(
        "Cooltrader",
        native_options,
        Box::new(|_| Box::new(cooltrader)),
    )
}


//debug zone

fn print_point(point: &PlotPoint){
    println!("x: {}, y: {}", point.x, point.y);
}

fn print_vector(vector: &PlotPoints){
    vector.points().iter().for_each(|x| print_point(x));
}

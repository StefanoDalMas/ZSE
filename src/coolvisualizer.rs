
//eframe related imports
use eframe::{App, CreationContext, egui, Frame, run_native};
use eframe::egui::{CentralPanel, Context};
use eframe::egui::plot::{Line, Plot, PlotPoints, PlotPoint};

//other imports
use rand::Rng;
use crate::filereader;


pub type data = egui::plot::PlotPoint; //might modify later idk

struct Cooltrader{
    name:String,
    dataset:filereader::Dataset,
}

impl Cooltrader{
    fn new() -> Self{
        Cooltrader{
            name:"Cooltrader".to_string(),
            dataset:filereader::Dataset::new(),
        }
    }
}
impl App for Cooltrader{
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| { //the window itself
            /*
            let sin: PlotPoints = (0..1000).map(|i| {

                let x = i as f64 * 0.01;
                [x, x.sin()]
            }).collect();
            let line = Line::new(sin);
            */
            let line = Line::new(self.dataset.return_plotpoints());
            //Plot::new("my_plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));
            let plot = Plot::new("cooltrader");
            plot.show(ui, |plot_ui| {
                plot_ui.line(line);
            });
        });
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

fn main(){
    let mut cooltrader = Cooltrader::new();
    let native_options = eframe::NativeOptions::default();

    cooltrader.dataset.append_vector(fake_data_generator()); //insert some data
    cooltrader.dataset.publish_5_times();
    let s = cooltrader.dataset.consume_one_line().unwrap();
    println!("caszzzo");
    println!("{}", s);
    run_native(
        "Cooltrader",
        native_options,
        Box::new(|_| Box::new(cooltrader)),
    )
}
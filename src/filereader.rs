use std::collections::VecDeque;
use eframe::egui::plot::{PlotPoint, PlotPoints};
use eframe::egui::plot::PlotPoints::Owned;

//other imports
use std::fs::{OpenOptions, write};
use std::io::{BufRead, BufReader, Read, Write};
use rand::Rng;
use crate::coolvisualizer;


//consts and types zone
const PATH_LOG :&str = "textdump.txt";

#[derive(Debug, Clone)]
pub(crate) struct Dataset{
    values :VecDeque<PlotPoint>,
    pub window_size: f64,
}

impl Dataset{
    pub fn new(window_size:f64) -> Self{
        init_file();
        Dataset{
            values: VecDeque::new(),
            window_size,
        }
    }
    pub fn append_single_plotpoint(&mut self, value:PlotPoint){
        //when I append a value I want to remove older ones
        let last_x = value.x - self.window_size;
        self.values.push_back(value);
        while let Some(el) = self.values.front(){
            if el.x < last_x{
                self.values.pop_front();
            }
            else{
                break;
            }
        }
    }
    pub fn append_vector(&mut self, values : Vec<PlotPoint>){
        for value in values{
            self.values.push_back(value);
        }
    }
    pub fn get_as_plotpoints(&self) -> PlotPoints{ //make dequeue into plotpoints
        Owned(Vec::from_iter(self.values.iter().cloned()))
    }

    pub fn publish_test_times(&mut self,times:i32){
        for _ in 0..times{
            write_random_metadata();
        }
    }
    pub fn consume_one_line(&mut self) -> Option<String>{
        consume_data()
    }

}



//test_file stuff
fn init_file() {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PATH_LOG);
    match file {
        Ok(file) => file,
        Err(_) => panic!("Error opening / creating file"),
    };
}

fn write_random_metadata() {
    let file = OpenOptions::new()
        .append(true)
        .open(PATH_LOG);
    match file {
        Ok(mut file) => {
            //generate random metadata
            let mut seed = rand::thread_rng();
            let eur = seed.gen_range(0.0..10000.0);
            let usd = seed.gen_range(0.0..10000.0);
            let yen = seed.gen_range(0.0..10000.0);
            let yuan = seed.gen_range(0.0..10000.0);
            let s = format!("EUR {} USD {} YEN {} YUAN {} \n",eur,usd,yen,yuan);
            let write = file.write_all(s.as_bytes());
            match write {
                Ok(_) => {}
                Err(_) => println!("Error writing to file"),
            }
        }
        Err(_) => panic!("Error opening file"),
    };
}

fn consume_data() -> Option<String>{
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(PATH_LOG);
    match file {
        Ok(file) => {
            //read a whole line
            let mut reader = BufReader::new(file);
            //read the line that I need
            let mut string = String::new();
            let res = reader.read_line(&mut string);
            match res {
                Ok(_) => {}
                Err(_) => println!("Error reading line"),
            }
            //get all lines and scroll them up by 1
            let mut lines = reader.lines()
                .map(|x| x.unwrap())
                .collect::<Vec<String>>().join("\n");
            //println!("HO AGGIUNTOOOOOO {}",lines);
            //check if all was ok
            let wrote = write(PATH_LOG, lines);
            match wrote {
                Ok(_) => {return Some(string)},
                Err(_) => {return None}
            }
        },
        Err(_) => {return None},
    };
}

#[test]
fn main(){
    let x = Dataset::new(coolvisualizer::WINSIZE);
}
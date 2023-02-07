use std::collections::VecDeque;
use eframe::egui::plot::{PlotPoint, PlotPoints};
use eframe::egui::plot::PlotPoints::Owned;

//other imports
use std::fs::{OpenOptions, write};
use std::io::{BufRead, BufReader, Read, Write};
use rand::Rng;


//consts and types zone
const PATH_LOG :&str = "textdump.txt";

#[derive(Debug, Clone)]
pub(crate) struct Dataset{
    values :VecDeque<PlotPoint>,
}

impl Dataset{
    pub fn new() -> Self{
        init_file();
        Dataset{
            values: VecDeque::new(),
        }
    }
    pub fn append_single_plotpoint(&mut self, value:PlotPoint){
        self.values.push_back(value);
    }
    pub fn append_vector(&mut self, values : Vec<PlotPoint>){
        for value in values{
            self.values.push_back(value);
        }
    }
    pub fn return_plotpoints(&self) -> PlotPoints{ //make dequeue into plotpoints
        Owned(Vec::from_iter(self.values.iter().cloned()))
    }

    pub fn publish_5_times(&mut self){
        for _ in 0..5{
            write_metadata();
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

fn write_metadata() {
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
            println!("HO AGGIUNTOOOOOO {}",lines);
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
    let x = Dataset::new();
}
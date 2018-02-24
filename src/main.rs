#[macro_use] extern crate structopt;

use std::collections::VecDeque;
use std::time::Instant;
use std::io::{StdinLock, Lines, BufRead, Write};


//
// Main `App`
//

#[derive(Debug)]
struct App {
    params: Params,
    hits: VecDeque<Instant>,
}

impl App {
    fn new() -> Self {
        use structopt::StructOpt;
        
        App {
            params: Params::from_args().validate(),
            hits: VecDeque::new(),
        }
    }
    
    fn display_bpm(&self, n: f64) {
        let (first, last) = (self.hits.front().unwrap(), self.hits.back().unwrap());
        let elapsed_time = last.duration_since(*first);
        let elapsed_millis = {
            let secs = elapsed_time.as_secs() as f64;
            let millisecs = (elapsed_time.subsec_nanos() / 1_000_000) as f64;
            millisecs + secs * 1_000.
        };
        let bpm = n * 60_000. / elapsed_millis;
        
        print!("Tempo: {:.*} bpm ", self.params.precision, bpm);
        std::io::stdout().flush().expect("Error: cannot flush");
    }
    
    fn reset_time_elapsed(&self, now: Instant) -> bool {        
        match self.hits.back() {
            Some(last) => now.duration_since(*last).as_secs() >= self.params.reset_time,
            None => false,
        }
    }
    
    fn run(&mut self) -> Result<(), String> {
        fn must_continue(reader: &mut Lines<StdinLock>) -> Result<bool, String> {
            reader
                .next()
                .unwrap_or(Ok("q".into()))
                .map_err(|e| format!("{}", e))
                .map(|s| s != "q")
        }

        let reader = std::io::stdin();
        let reader = &mut reader.lock().lines();

        println!("Hit enter key for each beat (q to quit).");

        while must_continue(reader)? {
            let now = Instant::now();
            
            if self.reset_time_elapsed(now) {
                self.hits.clear();
            }
            self.hits.push_back(now);
            
            match self.hits.len() {
                0 | 1 => println!("[Hit enter key one more time to start bpm computation...]"),
                n => {
                    if self.hits.len() > self.params.sample_size {
                        self.hits.pop_front();
                    }
                    self.display_bpm(n as f64)
                },
            }
        }
        println!("Bye Bye!");
        Ok(())
    }
}


//
// Main
//

fn main() {
    if let Err(e) = App::new().run() {
        println!("Error: {}", e);
        std::process::exit(-1);
    }
}


//
// Command line parameters
//

#[derive(Debug, StructOpt)]
struct Params {
    #[structopt(short = "p", long = "precision", default_value = "0")]
    /// set the decimal precision of the tempo display [max: 5]
    precision: usize,
    #[structopt(short = "r", long = "reset-time", default_value = "5")]
    /// set the time in second to reset the computation
    reset_time: u64,
    #[structopt(short = "s", long = "sample-size", default_value = "5")]
    /// set the number of samples needed to compute the tempo
    sample_size: usize,
}

impl Params {
    fn validate(self) -> Self {
        Params {
            precision: self.precision.min(5),
            reset_time: self.reset_time.max(1),
            sample_size: self.sample_size.max(1),
        }
    }
}


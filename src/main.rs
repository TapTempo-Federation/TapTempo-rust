#[macro_use] extern crate structopt;
extern crate circ_queue;

use std::time::Instant;
use std::io::{StdinLock, Lines, BufRead, Write, Error as IoError};
use circ_queue::CircQueue;


//
// print_now!
//

macro_rules! print_now {
    { $( $e:expr ),* } => {{
        print!( $( $e ),* );
        std::io::stdout().flush().expect("Error: cannot flush");
    }}
}


//
// Main `App`
//

#[derive(Debug)]
struct App {
    params: Params,
    hits: CircQueue<Instant>,
}

impl Default for App {
    fn default() -> Self {
        let params = Params::default();
        let hits = CircQueue::with_capacity(params.sample_size);
        
        App { params, hits }
    }
}

impl App {
    fn display_bpm(&self, n: f64) {
        let (first, last) = (self.hits.front().unwrap(), self.hits.back().unwrap());
        let elapsed_time = last.duration_since(*first);
        let elapsed_millis = {
            let secs = elapsed_time.as_secs() as f64;
            let millisecs = (elapsed_time.subsec_nanos() / 1_000_000) as f64;
            millisecs + secs * 1_000.
        };
        let bpm = n * 60_000. / elapsed_millis;
        
        print_now!("Tempo: {:.*} bpm ", self.params.precision, bpm);
    }
    
    fn reset_time_elapsed(&self, now: Instant) -> bool {        
        match self.hits.back() {
            Some(last) => now.duration_since(*last).as_secs() >= self.params.reset_time,
            None => false,
        }
    }
    
    fn run(&mut self) -> Result<(), Box<std::error::Error>> {
        fn must_continue(reader: &mut Lines<StdinLock>) -> Result<bool, IoError> {
            match reader.next() {
                None => Ok(false),
                Some(r) => r.map(|s| s != "q")
            }
        }

        let reader = std::io::stdin();
        let reader = &mut reader.lock().lines();

        print_now!("Hit enter key for each beat (q to quit).");

        while must_continue(reader)? {
            let now = Instant::now();
            
            if self.reset_time_elapsed(now) {
                self.hits.clear();
            }
            self.hits.push_back(now);
            
            match self.hits.len() {
                0 | 1 => print_now!("[Hit enter key one more time to start bpm computation...]"),
                n => self.display_bpm(n as f64),
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
    if let Err(e) = App::default().run() {
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

impl Default for Params {
    fn default() -> Self {
        use structopt::StructOpt;
        let params = Params::from_args();

        Params {
            precision: params.precision.min(5),
            reset_time: params.reset_time.max(1),
            sample_size: params.sample_size.max(1),
        }
    }
}

